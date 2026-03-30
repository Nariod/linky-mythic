package agent_functions

import agentstructs "github.com/MythicMeta/MythicContainerPkg/agent_structs"

func registerShell() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name:                "shell",
		Description:         "Execute a shell command via /bin/sh (Linux/macOS) or cmd.exe (Windows)",
		HelpString:          "shell <command>",
		Version:             1,
		Author:              "@your-handle",
		MitreAttackMappings: []string{"T1059"},
		SupportedUIFeatures: []string{"callback_table:shell"},
		CommandAttributes: agentstructs.CommandAttribute{
			SupportedOS: []agentstructs.OS{agentstructs.LINUX, agentstructs.WINDOWS, agentstructs.MACOS},
		},
		CommandParameters: []agentstructs.CommandParameter{
			{
				Name:             "command",
				ModalDisplayName: "Command to execute",
				CLIName:          "command",
				ParameterType:    agentstructs.COMMAND_PARAMETER_TYPE_STRING,
				Description:      "Shell command to run",
				Required:         true,
			},
		},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			resp := agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
			cmd, _ := taskData.Args.GetStringArg("command")
			resp.DisplayParams = &cmd
			return resp
		},
		TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
			return args.SetArgValue("command", input)
		},
	})
}
