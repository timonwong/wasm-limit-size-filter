package test

import (
	"os"
	"path/filepath"
	"runtime"
	"testing"
	"time"

	"istio.io/proxy/test/envoye2e/driver"
	"istio.io/proxy/testdata"
)

var testDir string

func init() {
	_, currentFile, _, _ := runtime.Caller(0)
	testDir = filepath.Dir(currentFile)
}

func TestLimitSize(t *testing.T) {
	wasmFile := filepath.Join(testDir, "../filter.wasm")

	tests := []struct {
		name           string
		wasmConfig     string
		requestBody    string
		responseCode   int
		requestChunked bool
		expectPanic    bool
	}{
		{
			name:        "WrongConfig",
			wasmConfig:  `{"abc": 123d}`,
			expectPanic: true,
		},
		{
			name:         "LimitRequest-1B-Fail",
			wasmConfig:   `{"maxRequestSize": 1}`,
			requestBody:  "hello",
			responseCode: 413,
		},
		{
			name:           "LimitRequest-5B-CT-Fail",
			wasmConfig:     `{"maxRequestSize": 5}`,
			requestBody:    "hello hello hello hello hello hello",
			responseCode:   413,
			requestChunked: true,
		},
		{
			name:         "LimitRequest-100B-Ok",
			wasmConfig:   `{"maxRequestSize": 100}`,
			requestBody:  "hello",
			responseCode: 200,
		},
		{
			name:         "LimitResponse-1B-Fail",
			wasmConfig:   `{"maxResponseSize": 1}`,
			responseCode: 502,
		},
		{
			name:         "LimitResponse-100B-Ok",
			wasmConfig:   `{"maxResponseSize": 100}`,
			responseCode: 200,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			params := driver.NewTestParams(t, map[string]string{
				"WasmConfig":          tt.wasmConfig,
				"WasmFile":            wasmFile,
				"ServerStaticCluster": loadTestData("testdata/cluster/static_server.yaml.tmpl"),
				"ServerHTTPFilters":   loadTestData("testdata/server_filter.yaml.tmpl"),
			}, ExtensionE2ETests)

			scenario := &driver.Scenario{
				Steps: []driver.Step{
					&driver.XDS{},
					&driver.Update{
						Node:    "server",
						Version: "0",
						Listeners: []string{
							string(testdata.MustAsset("listener/server.yaml.tmpl")),
							loadTestData("testdata/listener/staticreply.yaml.tmpl"),
						},
					},
					&driver.Envoy{
						Bootstrap:       params.FillTestData(string(testdata.MustAsset("bootstrap/server.yaml.tmpl"))),
						DownloadVersion: os.Getenv("ISTIO_TEST_VERSION"),
					},
					&driver.Sleep{Duration: 1 * time.Second},
					&HTTPCall{
						// request
						Port:               params.Ports.ServerPort,
						Method:             "POST",
						Path:               "/",
						RequestBody:        tt.requestBody,
						RequestChunked:     tt.requestChunked,
						RequestChunkedSize: 2,
						// expect
						ResponseCode: tt.responseCode,
					},
				},
			}

			if err := scenario.Run(params); err != nil {
				if !tt.expectPanic {
					t.Fatal(err)
				} else {
					t.Error(err)
				}
			}
		})
	}
}

func loadTestData(testFileName string) string {
	if !filepath.IsAbs(testFileName) {
		testFileName = filepath.Join(testDir, testFileName)
	}

	data, err := os.ReadFile(testFileName)
	if err != nil {
		panic(err)
	}

	return string(data)
}
