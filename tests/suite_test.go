package test

import (
	"io/ioutil"
	"os"
	"path/filepath"
	"testing"
	"time"

	"istio.io/proxy/test/envoye2e/driver"
	"istio.io/proxy/test/envoye2e/env"
	"istio.io/proxy/testdata"
)

// Inventory is initialised at the bottom of the file. Port assignments will be
// offset based on the index of the test in the inventory.
var ExtensionE2ETests *env.TestInventory

func init() {
	ExtensionE2ETests = &env.TestInventory{
		Tests: []string{
			"TestAddHeader/NoConfig",
			"TestAddHeader/WrongConfig",
			"TestAddHeader/ExtraHeaders",
		},
	}
}

func TestAddHeader(t *testing.T) {
	wasmFile, err := filepath.Abs("../extension.wasm")
	if err != nil {
		panic(err)
	}

	tests := []struct {
		name            string
		wasmConfig      string
		responseHeaders map[string]string
	}{
		{
			name: "NoConfig",
			responseHeaders: map[string]string{
				"wa-demo":      "true",
				"x-powered-by": "add-header-rs",
				"x-header-1":   "-",
				"x-header-2":   "-",
			},
		},
		{
			name:       "WrongConfig",
			wasmConfig: `{"abc": 123d}`,
			responseHeaders: map[string]string{
				"wa-demo":      "true",
				"x-powered-by": "add-header-rs",
				"x-header-1":   "-",
				"x-header-2":   "-",
			},
		},
		{
			name:       "ExtraHeaders",
			wasmConfig: `{"x-header-1":"header one","x-header-2":"header two"}`,
			responseHeaders: map[string]string{
				"wa-demo":      "true",
				"x-powered-by": "add-header-rs",
				"x-header-1":   "header one",
				"x-header-2":   "header two",
			},
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
					&driver.HTTPCall{
						// request
						Method: "GET",
						Path:   "/",
						Port:   params.Ports.ServerPort,
						// expect
						ResponseCode:    200,
						ResponseHeaders: tt.responseHeaders,
					},
				},
			}

			if err := scenario.Run(params); err != nil {
				t.Fatal(err)
			}
		})
	}
}

func loadTestData(testFileName string) string {
	data, err := ioutil.ReadFile(testFileName)
	if err != nil {
		panic(err)
	}

	return string(data)
}
