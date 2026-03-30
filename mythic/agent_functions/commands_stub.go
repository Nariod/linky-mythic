package agent_functions

import agentstructs "github.com/MythicMeta/MythicContainerPkg/agent_structs"

func registerLs() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "ls", Description: "List directory contents", HelpString: "ls [path]", Version: 1,
		MitreAttackMappings: []string{"T1083"},
		CommandAttributes:   agentstructs.CommandAttribute{SupportedOS: []agentstructs.OS{agentstructs.LINUX, agentstructs.WINDOWS, agentstructs.MACOS}},
		CommandParameters: []agentstructs.CommandParameter{
			{
				Name: "path", CLIName: "path",
				ModalDisplayName: "Directory path",
				ParameterType:    agentstructs.COMMAND_PARAMETER_TYPE_STRING,
				Description:      "Directory to list (default: current directory)",
				Required:         false, DefaultValue: ".",
			},
		},
		TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
			if input == "" {
				input = "."
			}
			return args.SetArgValue("path", input)
		},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			resp := agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
			path, _ := taskData.Args.GetStringArg("path")
			resp.DisplayParams = &path
			return resp
		},
	})
}

func registerCd() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "cd", Description: "Change working directory", HelpString: "cd <path>", Version: 1,
		CommandAttributes: agentstructs.CommandAttribute{SupportedOS: []agentstructs.OS{agentstructs.LINUX, agentstructs.WINDOWS, agentstructs.MACOS}},
		CommandParameters: []agentstructs.CommandParameter{
			{
				Name: "path", CLIName: "path",
				ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_STRING,
				Required:      true,
			},
		},
		TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
			return args.SetArgValue("path", input)
		},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			resp := agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
			path, _ := taskData.Args.GetStringArg("path")
			resp.DisplayParams = &path
			return resp
		},
	})
}

func registerPwd() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "pwd", Description: "Print working directory", HelpString: "pwd", Version: 1,
		CommandAttributes: agentstructs.CommandAttribute{SupportedOS: []agentstructs.OS{agentstructs.LINUX, agentstructs.WINDOWS, agentstructs.MACOS}},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			return agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
		},
	})
}

func registerWhoami() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "whoami", Description: "Current user identity", HelpString: "whoami", Version: 1,
		MitreAttackMappings: []string{"T1033"},
		CommandAttributes:   agentstructs.CommandAttribute{SupportedOS: []agentstructs.OS{agentstructs.LINUX, agentstructs.WINDOWS, agentstructs.MACOS}},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			return agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
		},
	})
}

func registerPid() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "pid", Description: "Current process ID", HelpString: "pid", Version: 1,
		CommandAttributes: agentstructs.CommandAttribute{SupportedOS: []agentstructs.OS{agentstructs.LINUX, agentstructs.WINDOWS, agentstructs.MACOS}},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			return agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
		},
	})
}

func registerInfo() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "info", Description: "System information", HelpString: "info", Version: 1,
		MitreAttackMappings: []string{"T1082"},
		CommandAttributes:   agentstructs.CommandAttribute{SupportedOS: []agentstructs.OS{agentstructs.LINUX, agentstructs.WINDOWS, agentstructs.MACOS}},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			return agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
		},
	})
}

func registerPs() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "ps", Description: "List running processes", HelpString: "ps", Version: 1,
		MitreAttackMappings: []string{"T1057"},
		CommandAttributes:   agentstructs.CommandAttribute{SupportedOS: []agentstructs.OS{agentstructs.LINUX, agentstructs.WINDOWS, agentstructs.MACOS}},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			return agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
		},
	})
}

func registerNetstat() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "netstat", Description: "List network connections", HelpString: "netstat", Version: 1,
		MitreAttackMappings: []string{"T1049"},
		CommandAttributes:   agentstructs.CommandAttribute{SupportedOS: []agentstructs.OS{agentstructs.LINUX, agentstructs.WINDOWS, agentstructs.MACOS}},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			return agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
		},
	})
}

func registerDownload() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "download", Description: "Download a file from the implant", HelpString: "download <remote_path>", Version: 1,
		MitreAttackMappings: []string{"T1041"},
		CommandAttributes:   agentstructs.CommandAttribute{SupportedOS: []agentstructs.OS{agentstructs.LINUX, agentstructs.WINDOWS, agentstructs.MACOS}},
		CommandParameters: []agentstructs.CommandParameter{
			{
				Name: "path", CLIName: "path",
				ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_STRING,
				Required:      true,
			},
		},
		TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
			return args.SetArgValue("path", input)
		},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			resp := agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
			path, _ := taskData.Args.GetStringArg("path")
			resp.DisplayParams = &path
			return resp
		},
	})
}

func registerUpload() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "upload", Description: "Upload a file to the implant", HelpString: "upload <local> <remote>", Version: 1,
		MitreAttackMappings: []string{"T1105"},
		CommandAttributes:   agentstructs.CommandAttribute{SupportedOS: []agentstructs.OS{agentstructs.LINUX, agentstructs.WINDOWS, agentstructs.MACOS}},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			return agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
		},
	})
}

func registerSleep() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "sleep", Description: "Set sleep interval and jitter", HelpString: "sleep <seconds> [jitter%]", Version: 1,
		CommandAttributes: agentstructs.CommandAttribute{SupportedOS: []agentstructs.OS{agentstructs.LINUX, agentstructs.WINDOWS, agentstructs.MACOS}},
		CommandParameters: []agentstructs.CommandParameter{
			{
				Name: "args", CLIName: "args",
				ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_STRING,
				Required:      true,
			},
		},
		TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
			return args.SetArgValue("args", input)
		},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			resp := agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
			a, _ := taskData.Args.GetStringArg("args")
			resp.DisplayParams = &a
			return resp
		},
	})
}

func registerKilldate() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "killdate", Description: "Set auto-exit date", HelpString: "killdate <timestamp|clear>", Version: 1,
		CommandAttributes: agentstructs.CommandAttribute{SupportedOS: []agentstructs.OS{agentstructs.LINUX, agentstructs.WINDOWS, agentstructs.MACOS}},
		CommandParameters: []agentstructs.CommandParameter{
			{
				Name: "date", CLIName: "date",
				ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_STRING,
				Required:      true,
			},
		},
		TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
			return args.SetArgValue("date", input)
		},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			resp := agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
			d, _ := taskData.Args.GetStringArg("date")
			resp.DisplayParams = &d
			return resp
		},
	})
}

func registerInject() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "inject", Description: "Inject base64 shellcode into PID (Windows)", HelpString: "inject <pid> <base64_shellcode>", Version: 1,
		MitreAttackMappings: []string{"T1055"},
		CommandAttributes:   agentstructs.CommandAttribute{SupportedOS: []agentstructs.OS{agentstructs.WINDOWS}},
		CommandParameters: []agentstructs.CommandParameter{
			{
				Name: "args", CLIName: "args",
				ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_STRING,
				Required:      true,
			},
		},
		TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
			return args.SetArgValue("args", input)
		},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			return agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
		},
	})
}

func registerIntegrity() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "integrity", Description: "Query token integrity level (Windows)", HelpString: "integrity", Version: 1,
		MitreAttackMappings: []string{"T1134"},
		CommandAttributes:   agentstructs.CommandAttribute{SupportedOS: []agentstructs.OS{agentstructs.WINDOWS}},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			return agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
		},
	})
}
