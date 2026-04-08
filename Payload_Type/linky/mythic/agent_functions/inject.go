package agent_functions

import (
	"encoding/json"
	"fmt"
	"strconv"
	"strings"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

func registerInject() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "inject", Description: "Inject base64 shellcode into a process (Windows)", HelpString: "inject <pid> <base64_shellcode>", Version: 1,
		MitreAttackMappings: []string{"T1055"},
		CommandAttributes:   agentstructs.CommandAttribute{SupportedOS: []string{agentstructs.SUPPORTED_OS_WINDOWS}},
		CommandParameters: []agentstructs.CommandParameter{
			{
				Name: "pid", CLIName: "pid",
				ParameterType:            agentstructs.COMMAND_PARAMETER_TYPE_NUMBER,
				Description:              "Target process PID",
				ParameterGroupInformation: []agentstructs.ParameterGroupInfo{{ParameterIsRequired: true, GroupName: "Default"}},
			},
			{
				Name: "shellcode", CLIName: "shellcode",
				ParameterType:            agentstructs.COMMAND_PARAMETER_TYPE_STRING,
				Description:              "Base64-encoded shellcode payload",
				ParameterGroupInformation: []agentstructs.ParameterGroupInfo{{ParameterIsRequired: true, GroupName: "Default"}},
			},
		},
		TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
			var jsonArgs map[string]interface{}
			if err := json.Unmarshal([]byte(input), &jsonArgs); err == nil {
				return args.LoadArgsFromJSONString(input)
			}
			parts := strings.SplitN(strings.TrimSpace(input), " ", 2)
			if len(parts) != 2 {
				return fmt.Errorf("usage: inject <pid> <base64_shellcode>")
			}
			pid, err := strconv.ParseFloat(parts[0], 64)
			if err != nil {
				return fmt.Errorf("invalid PID %q: %w", parts[0], err)
			}
			if err := args.SetArgValue("pid", pid); err != nil {
				return err
			}
			return args.SetArgValue("shellcode", parts[1])
		},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			resp := agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
			pid, _ := taskData.Args.GetNumberArg("pid")
			display := fmt.Sprintf("pid=%.0f", pid)
			resp.DisplayParams = &display
			return resp
		},
	})
}
