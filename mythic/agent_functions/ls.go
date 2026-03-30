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