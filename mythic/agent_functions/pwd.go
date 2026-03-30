package agent_functions

import agentstructs "github.com/MythicMeta/MythicContainerPkg/agent_structs"

func registerPwd() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "pwd", Description: "Print working directory", HelpString: "pwd", Version: 1,
		CommandAttributes: agentstructs.CommandAttribute{SupportedOS: []agentstructs.OS{agentstructs.LINUX, agentstructs.WINDOWS, agentstructs.MACOS}},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			return agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
		},
	})
}
