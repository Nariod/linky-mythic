package agent_functions

import (
	"context"
	"encoding/json"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

func registerBof() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name:        "bof",
		Description: "Load and execute a Beacon Object File (COFF) in-memory. Args use the bof_pack format: int:<n> short:<n> str:<s> wstr:<s> bin:<base64>",
		HelpString:  "bof <file> [entrypoint] [args]",
		Version:     1,
		MitreAttackMappings: []string{"T1059", "T1105"},
		CommandAttributes:   agentstructs.CommandAttribute{SupportedOS: []string{agentstructs.SUPPORTED_OS_WINDOWS}},
		CommandParameters: []agentstructs.CommandParameter{
			{
				Name:        "bof",
				CLIName:     "bof",
				ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_FILE,
				Description:   "Beacon Object File (.x64.o / .o COFF object)",
				ParameterGroupInformation: []agentstructs.ParameterGroupInfo{
					{ParameterIsRequired: true, GroupName: "Default"},
				},
			},
			{
				Name:        "entrypoint",
				CLIName:     "entrypoint",
				ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_STRING,
				Description:   "BOF entrypoint symbol (default: go)",
				ParameterGroupInformation: []agentstructs.ParameterGroupInfo{
					{ParameterIsRequired: false, GroupName: "Default"},
				},
			},
			{
				Name:        "args",
				CLIName:     "args",
				ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_STRING,
				Description:   "Arguments in bof_pack format, space-separated: int:<n> short:<n> str:<s> wstr:<s> bin:<base64>",
				ParameterGroupInformation: []agentstructs.ParameterGroupInfo{
					{ParameterIsRequired: false, GroupName: "Default"},
				},
			},
		},
		TaskFunctionParseArgString: func(ctx context.Context, args *agentstructs.PTTaskMessageArgsData, input string) error {
			var jsonArgs map[string]interface{}
			if err := json.Unmarshal([]byte(input), &jsonArgs); err == nil {
				return args.LoadArgsFromJSONString(input)
			}
			parts := splitArgs(input, 3)
			if len(parts) == 0 {
				return nil
			}
			if err := args.SetArgValue("bof", parts[0]); err != nil {
				return err
			}
			if len(parts) > 1 {
				if err := args.SetArgValue("entrypoint", parts[1]); err != nil {
					return err
				}
			}
			if len(parts) > 2 {
				return args.SetArgValue("args", parts[2])
			}
			return nil
		},
		TaskFunctionCreateTasking: func(ctx context.Context, taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			resp := agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
			if entrypoint, err := taskData.Args.GetStringArg("entrypoint"); err == nil && entrypoint != "" {
				resp.DisplayParams = &entrypoint
			}
			return resp
		},
	})
}
