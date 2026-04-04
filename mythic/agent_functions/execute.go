package agent_functions

import (
	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

func registerExecute() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "execute", Description: "Execute a binary directly (no shell)", HelpString: "execute <binary> [args...]", Version: 1,
		CommandAttributes: agentstructs.CommandAttribute{SupportedOS: []string{agentstructs.SUPPORTED_OS_LINUX, agentstructs.SUPPORTED_OS_WINDOWS, agentstructs.SUPPORTED_OS_MACOS}},
		CommandParameters: []agentstructs.CommandParameter{
			{
				Name: "command", CLIName: "command",
				ParameterType:             agentstructs.COMMAND_PARAMETER_TYPE_STRING,
				ParameterGroupInformation: []agentstructs.ParameterGroupInfo{{ParameterIsRequired: true, GroupName: "Default"}},
				Description:               "Binary path and arguments (e.g. /usr/bin/id --name)",
			},
		},
		TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
			return args.SetArgValue("command", input)
		},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			return agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
		},
	})
}
