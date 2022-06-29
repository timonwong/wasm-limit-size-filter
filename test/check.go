package test

import (
	"bufio"
	"fmt"
	"io"
	"log"
	"net/http"
	"net/http/httputil"
	"strings"
	"time"

	"istio.io/proxy/test/envoye2e/driver"
)

// HTTPCall sends a HTTP request to a localhost port, and then check the response code, and response headers.
type HTTPCall struct {
	// Method
	Method string
	// URL path
	Path string
	// Port specifies the port in 127.0.0.1:PORT
	Port uint16
	// Body is the expected body
	Body string
	// RequestChunked uses Transfer-Encoding: chunked
	RequestChunked     bool
	RequestChunkedSize int
	// RequestBody to send with the request
	RequestBody string
	// RequestHeaders to send with the request
	RequestHeaders map[string]string
	// ResponseCode to expect
	ResponseCode int
	// ResponseHeaders to expect
	ResponseHeaders map[string]string
	// Timeout (must be set to avoid the default)
	Timeout time.Duration
}

func (g *HTTPCall) Run(_ *driver.Params) error {
	url := fmt.Sprintf("http://127.0.0.1:%d%v", g.Port, g.Path)
	if g.Timeout == 0 {
		g.Timeout = driver.DefaultTimeout
	}

	var reqBody io.Reader
	if len(g.RequestBody) > 0 {
		reqBody = strings.NewReader(g.RequestBody)

		if g.RequestChunked {
			chunkSize := 1024
			if g.RequestChunkedSize > 0 {
				chunkSize = g.RequestChunkedSize
			}

			reqBody = bufio.NewReaderSize(reqBody, chunkSize)
		}
	}

	req, err := http.NewRequest(g.Method, url, reqBody)
	if err != nil {
		return err
	}

	if g.RequestChunked {
		req.ContentLength = -1
	}

	for key, val := range g.RequestHeaders {
		req.Header.Add(key, val)
	}

	dump, _ := httputil.DumpRequest(req, false)
	log.Printf("HTTP request:\n%s", string(dump))

	client := &http.Client{Timeout: g.Timeout}
	resp, err := client.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	code := resp.StatusCode
	wantCode := 200
	if g.ResponseCode != 0 {
		wantCode = g.ResponseCode
	}
	if code != wantCode {
		return fmt.Errorf("error code for :%d: %d", g.Port, code)
	}

	bodyBytes, err := io.ReadAll(resp.Body)
	if err != nil {
		return err
	}

	body := string(bodyBytes)
	if g.Body != "" && g.Body != body {
		return fmt.Errorf("got body %q, want %q", body, g.Body)
	}

	for key, val := range g.ResponseHeaders {
		got := resp.Header.Get(key)
		switch val {
		case driver.Any:
			if got == "" {
				return fmt.Errorf("got response header %q, want any", got)
			}
		case driver.None:
			if got != "" {
				return fmt.Errorf("got response header %q, want none", got)
			}
		default:
			if got != val {
				return fmt.Errorf("got response header %q, want %q", got, val)
			}
		}
	}

	return nil
}
func (g *HTTPCall) Cleanup() {}
