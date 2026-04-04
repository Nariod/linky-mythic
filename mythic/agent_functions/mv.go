package agent_functions

import (
	"fmt"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

func registerMv() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "mv", Description: "Move or rename a file or directory", HelpString: "mv <source> <destination>", Version: 1,
		CommandAttributes: agentstructs.CommandAttribute{SupportedOS: []string{agentstructs.SUPPORTED_OS_LINUX, agentstructs.SUPPORTED_OS_WINDOWS, agentstructs.SUPPORTED_OS_MACOS}},
		CommandParameters: []agentstructs.CommandParameter{
			{
				Name: "source", CLIName: "source",
				ParameterType:             agentstructs.COMMAND_PARAMETER_TYPE_STRING,
				ParameterGroupInformation: []agentstructs.ParameterGroupInfo{{ParameterIsRequired: true, GroupName: "Default"}},
				Description:               "Source file or directory path",
			},
			{
				Name: "destination", CLIName: "destination",
				ParameterType:             agentstructs.COMMAND_PARAMETER_TYPE_STRING,
				ParameterGroupInformation: []agentstructs.ParameterGroupInfo{{ParameterIsRequired: true, GroupName: "Default"}},
				Description:               "Destination path",
			},
		},
		TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
			parts := splitArgs(input, 2)
			if len(parts) < 2 {
				return fmt.Errorf("usage: mv <source> <destination>")
			}
			if err := args.SetArgValue("source", parts[0]); err != nil {
				return err
			}
			return args.SetArgValue("destination", parts[1])
		},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			return agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
		},
	})
}
