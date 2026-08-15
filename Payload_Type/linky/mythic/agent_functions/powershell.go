package agent_functions

import agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"

func registerPowershell() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name:                "powershell",
		Description:         "Execute a command via powershell.exe -noP -sta -w 1 -c (Windows only)",
		HelpString:          "powershell <command>",
		Version:             1,
		Author:              "@Nariod",
		MitreAttackMappings: []string{"T1059"},
		CommandAttributes: agentstructs.CommandAttribute{
			SupportedOS: []string{agentstructs.SUPPORTED_OS_WINDOWS},
		},
		CommandParameters: []agentstructs.CommandParameter{
			{
				Name:             "command",
				ModalDisplayName: "PowerShell command to execute",
				CLIName:          "command",
				ParameterType:    agentstructs.COMMAND_PARAMETER_TYPE_STRING,
				Description:      "PowerShell command to run",
				ParameterGroupInformation: []agentstructs.ParameterGroupInfo{{ParameterIsRequired: true, GroupName: "Default"}},
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
