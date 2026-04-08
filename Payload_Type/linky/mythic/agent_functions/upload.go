package agent_functions

import (
	"encoding/json"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

func registerUpload() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "upload", Description: "Upload a file from Mythic to the target", HelpString: "upload <remote_path>", Version: 1,
		MitreAttackMappings: []string{"T1105"},
		CommandAttributes:   agentstructs.CommandAttribute{SupportedOS: []string{agentstructs.SUPPORTED_OS_LINUX, agentstructs.SUPPORTED_OS_WINDOWS, agentstructs.SUPPORTED_OS_MACOS}},
		CommandParameters: []agentstructs.CommandParameter{
			{
				Name: "file", CLIName: "file",
				ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_FILE,
				Description:   "File to upload to the target",
				ParameterGroupInformation: []agentstructs.ParameterGroupInfo{
					{ParameterIsRequired: true, GroupName: "Default"},
				},
			},
			{
				Name: "remote_path", CLIName: "remote_path",
				ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_STRING,
				Description:   "Destination path on the target",
				ParameterGroupInformation: []agentstructs.ParameterGroupInfo{
					{ParameterIsRequired: true, GroupName: "Default"},
				},
			},
		},
		TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
			var jsonArgs map[string]interface{}
			if err := json.Unmarshal([]byte(input), &jsonArgs); err == nil {
				return args.LoadArgsFromJSONString(input)
			}
			return args.SetArgValue("remote_path", input)
		},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			resp := agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
			remotePath, _ := taskData.Args.GetStringArg("remote_path")
			resp.DisplayParams = &remotePath
			return resp
		},
	})
}
