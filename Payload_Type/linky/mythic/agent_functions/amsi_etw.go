package agent_functions

import (
	"context"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

func registerAmsiEtw() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name:                "amsi_etw",
		Description:         "Patch AMSI (AmsiScanBuffer) and ETW (EtwEventWrite) in-memory for the current process (Windows only)",
		HelpString:          "amsi_etw",
		Version:             1,
		Author:              "@Nariod",
		MitreAttackMappings: []string{"T1562.001"},
		CommandAttributes: agentstructs.CommandAttribute{
			SupportedOS: []string{agentstructs.SUPPORTED_OS_WINDOWS},
		},
		TaskFunctionCreateTasking: func(ctx context.Context, taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			resp := agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
			return resp
		},
		TaskFunctionParseArgString: func(ctx context.Context, args *agentstructs.PTTaskMessageArgsData, input string) error {
			return nil
		},
	})
}
