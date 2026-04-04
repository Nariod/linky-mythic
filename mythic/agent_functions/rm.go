package agent_functions

import (
	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

func registerRm() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "rm", Description: "Remove a file or directory (recursive)", HelpString: "rm <path>", Version: 1,
		CommandAttributes: agentstructs.CommandAttribute{SupportedOS: []string{agentstructs.SUPPORTED_OS_LINUX, agentstructs.SUPPORTED_OS_WINDOWS, agentstructs.SUPPORTED_OS_MACOS}},
		CommandParameters: []agentstructs.CommandParameter{
			{
				Name: "path", CLIName: "path",
				ParameterType:             agentstructs.COMMAND_PARAMETER_TYPE_STRING,
				ParameterGroupInformation: []agentstructs.ParameterGroupInfo{{ParameterIsRequired: true, GroupName: "Default"}},
				Description:               "File or directory to remove",
			},
		},
		TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
			return args.SetArgValue("path", input)
		},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			return agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
		},
	})
}
