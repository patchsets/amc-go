package solver

import (
	"bufio"
	"bytes"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"strings"
	"sync"
	"sync/atomic"
	"time"
)

var (
	proxies    []string
	proxyIndex uint64
	proxyOnce  sync.Once
)

func loadProxies() {
	proxyOnce.Do(func() {
		f, err := os.Open("solving_proxies.txt")
		if err != nil {
			log.Fatalf("Failed to open proxies.txt: %v", err)
		}
		defer f.Close()

		scanner := bufio.NewScanner(f)
		for scanner.Scan() {
			line := strings.TrimSpace(scanner.Text())
			if line != "" {
				proxies = append(proxies, line)
			}
		}
		if len(proxies) == 0 {
			log.Fatal("proxies.txt is empty")
		}
		log.Printf("Loaded %d proxies", len(proxies))
	})
}

func nextProxy() string {
	idx := atomic.AddUint64(&proxyIndex, 1) - 1
	return proxies[idx%uint64(len(proxies))]
}

type TurnstileResponse struct {
	Success bool   `json:"success"`
	Token   string `json:"result"`
	Error   string `json:"error"`
}

func SolveTurnstile() (string, error) {
	loadProxies()
	proxy := nextProxy()

	payload := map[string]string{
		"type":     "turnstile",
		"site_key": "0x4AAAAAAA9oPHboisPr8cag",
		"url":      "https://graph.amctheatres.com/",
		"action":   "login",
		"proxy":    proxy,
	}

	body, err := json.Marshal(payload)
	if err != nil {
		return "", fmt.Errorf("failed to marshal payload: %w", err)
	}

	req, err := http.NewRequest("POST", "http://38.22.104.125:3003/turnstile_solve", bytes.NewReader(body))
	if err != nil {
		return "", fmt.Errorf("failed to build request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-API-Key", "nsl_d93dfc0eebe91ad945bec902f57e4e4ec85fe6af018352d4")

	httpClient := &http.Client{Timeout: 30 * time.Second}
	resp, err := httpClient.Do(req)
	if err != nil {
		return "", fmt.Errorf("request failed: %w", err)
	}
	defer resp.Body.Close()

	var result TurnstileResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return "", fmt.Errorf("failed to decode response: %w", err)
	}

	if !result.Success {
		return "", fmt.Errorf("solver error: %s", result.Error)
	}

	return result.Token, nil
}
