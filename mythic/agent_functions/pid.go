package agent_functions

import agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"

func registerPid() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "pid", Description: "Current process ID", HelpString: "pid", Version: 1,
		CommandAttributes: agentstructs.CommandAttribute{SupportedOS: []string{agentstructs.SUPPORTED_OS_LINUX, agentstructs.SUPPORTED_OS_WINDOWS, agentstructs.SUPPORTED_OS_MACOS}},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			return agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
		},
	})
}
