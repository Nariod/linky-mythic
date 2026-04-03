package agent_functions

import agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"

func registerNetstat() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "netstat", Description: "List network connections", HelpString: "netstat", Version: 1,
		MitreAttackMappings: []string{"T1049"},
		CommandAttributes:   agentstructs.CommandAttribute{SupportedOS: []string{agentstructs.SUPPORTED_OS_LINUX, agentstructs.SUPPORTED_OS_WINDOWS, agentstructs.SUPPORTED_OS_MACOS}},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			return agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
		},
	})
}
