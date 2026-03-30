package agent_functions

import agentstructs "github.com/MythicMeta/MythicContainerPkg/agent_structs"

func registerNetstat() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "netstat", Description: "List network connections", HelpString: "netstat", Version: 1,
		MitreAttackMappings: []string{"T1049"},
		CommandAttributes:   agentstructs.CommandAttribute{SupportedOS: []agentstructs.OS{agentstructs.LINUX, agentstructs.WINDOWS, agentstructs.MACOS}},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			return agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
		},
	})
}
