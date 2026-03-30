package agent_functions

import agentstructs "github.com/MythicMeta/MythicContainerPkg/agent_structs"

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
