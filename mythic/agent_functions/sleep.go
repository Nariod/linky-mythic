package agent_functions

import (
	"fmt"

	agentstructs "github.com/MythicMeta/MythicContainerPkg/agent_structs"
)

func registerSleep() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "sleep", Description: "Set sleep interval and jitter", HelpString: "sleep <seconds> [jitter%]", Version: 1,
		CommandAttributes: agentstructs.CommandAttribute{SupportedOS: []agentstructs.OS{agentstructs.LINUX, agentstructs.WINDOWS, agentstructs.MACOS}},
		CommandParameters: []agentstructs.CommandParameter{
			{
				Name: "seconds", CLIName: "seconds",
				ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_NUMBER,
				Required:      true,
			},
			{
				Name: "jitter", CLIName: "jitter",
				ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_NUMBER,
				Required:      false,
				DefaultValue:  0,
			},
		},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			resp := agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
			seconds, _ := taskData.Args.GetNumberArg("seconds")
			jitter, _ := taskData.Args.GetNumberArg("jitter")
			display := fmt.Sprintf("%.0fs jitter=%.0f%%", seconds, jitter)
			resp.DisplayParams = &display
			return resp
		},
	})
}
