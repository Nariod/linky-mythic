package agent_functions

import agentstructs "github.com/MythicMeta/MythicContainerPkg/agent_structs"

func registerInfo() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "info", Description: "System information", HelpString: "info", Version: 1,
		MitreAttackMappings: []string{"T1082"},
		CommandAttributes:   agentstructs.CommandAttribute{SupportedOS: []agentstructs.OS{agentstructs.LINUX, agentstructs.WINDOWS, agentstructs.MACOS}},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			return agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
		},
	})
}