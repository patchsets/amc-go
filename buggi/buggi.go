package buggi

import (
	"os"
	"strings"
	"sync"
	"time"
	"unsafe"

	"github.com/sqweek/dialog"
	"golang.org/x/sys/windows"
)

var (
	once      sync.Once
	blackList []string
	quit      chan struct{}

	ntdll                          = windows.NewLazySystemDLL("ntdll.dll")
	kernel32                       = windows.NewLazySystemDLL("kernel32.dll")
	procIsDebuggerPresent          = kernel32.NewProc("IsDebuggerPresent")
	procCheckRemoteDebuggerPresent = kernel32.NewProc("CheckRemoteDebuggerPresent")
	procNtQueryInformationProcess  = ntdll.NewProc("NtQueryInformationProcess")
)

func init() {
	once.Do(func() {
		blackList = []string{
			"ollydbg",
			"ida",
			"ida64",
			"idag",
			"idag64",
			"idaw",
			"idaw64",
			"idaq",
			"idaq64",
			"idau",
			"idau64",
			"scylla",
			"protection_id",
			"x64dbg",
			"x32dbg",
			"windbg",
			"reshacker",
			"importrec",
			"immunitydebugger",
			"megadumper",
			"cheatengine",
			"cheat engine",
			"wireshark",
			"dumpcap",
			"tshark",
			"rawshark",
			"httpdebugger",
			"httpdebuggerpro",
			"fiddler",
			"charles",
			"processhacker",
			"process hacker",
			"procmon",
			"procexp",
			"procexp64",
			"dnspy",
			"de4dot",
			"ilspy",
			"dotpeek",
			"ghidra",
			"hxd",
			"pestudio",
			"autoruns",
			"burpsuite",
			"proxifier",
			"tcpdump",
			"reclass",
			"simpleassemblyexplorer",
			"ildasm",
			"graywolf",
			"mdbg",
			"regmon",
			"filemon",
			"lordpe",
			"sysinternals",
			"apitrace",
			"apimonitor",
			"api monitor",
		}
	})
}

func isDebuggerPresent() bool {
	ret, _, _ := procIsDebuggerPresent.Call()
	return ret != 0
}

func checkRemoteDebugger() bool {
	var debuggerPresent int32
	handle := windows.CurrentProcess()
	ret, _, _ := procCheckRemoteDebuggerPresent.Call(
		uintptr(handle),
		uintptr(unsafe.Pointer(&debuggerPresent)),
	)
	return ret != 0 && debuggerPresent != 0
}

func checkDebugPort() bool {
	var debugPort uintptr
	handle := windows.CurrentProcess()
	status, _, _ := procNtQueryInformationProcess.Call(
		uintptr(handle),
		7, // ProcessDebugPort
		uintptr(unsafe.Pointer(&debugPort)),
		unsafe.Sizeof(debugPort),
		0,
	)
	return status == 0 && debugPort != 0
}

func SimpleRun(delay int) {
	ticker := time.NewTicker(time.Duration(delay) * time.Second)
	quit = make(chan struct{})
	go func() {
		for {
			select {
			case <-ticker.C:
				DetectAndClose()
			case <-quit:
				ticker.Stop()
				return
			}
		}
	}()
}

func HeartbeatCheckingTest() {
	if n := checkAll(); n != "" {
		dialog.Message("Detected: %s", n).Title("Anti debug alert").YesNo()
		os.Exit(0)
	}
}

func DetectAndClose() {
	if n := checkAll(); n != "" {
		os.Exit(0)
	}
}

func DetectAndReturn() string {
	return checkAll()
}

func checkAll() string {
	if isDebuggerPresent() {
		return "debugger (IsDebuggerPresent)"
	}
	if checkRemoteDebugger() {
		return "remote debugger"
	}
	if checkDebugPort() {
		return "debug port"
	}
	return checkProcess()
}

func checkProcess() string {
	snapshot, err := windows.CreateToolhelp32Snapshot(windows.TH32CS_SNAPPROCESS, 0)
	if err != nil {
		return ""
	}
	defer windows.CloseHandle(snapshot)

	var entry windows.ProcessEntry32
	entry.Size = uint32(unsafe.Sizeof(entry))

	err = windows.Process32First(snapshot, &entry)
	if err != nil {
		return ""
	}

	for {
		name := windows.UTF16ToString(entry.ExeFile[:])
		nameLower := strings.ToLower(name)

		for _, bl := range blackList {
			if strings.Contains(nameLower, bl) {
				return name
			}
		}

		err = windows.Process32Next(snapshot, &entry)
		if err != nil {
			break
		}
	}

	return ""
}
