package agent_functions

import agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"

func registerLs() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "ls", Description: "List directory contents (populates Mythic file browser)", HelpString: "ls [path]", Version: 1,
		MitreAttackMappings: []string{"T1083"},
		CommandAttributes:   agentstructs.CommandAttribute{SupportedOS: []string{agentstructs.SUPPORTED_OS_LINUX, agentstructs.SUPPORTED_OS_WINDOWS, agentstructs.SUPPORTED_OS_MACOS}},
		CommandParameters: []agentstructs.CommandParameter{
			{
				Name: "path", CLIName: "path",
				ModalDisplayName: "Directory path",
				ParameterType:    agentstructs.COMMAND_PARAMETER_TYPE_STRING,
				Description:      "Directory to list (default: current directory)",
				DefaultValue:     ".",
				ParameterGroupInformation: []agentstructs.ParameterGroupInfo{{ParameterIsRequired: false, GroupName: "Default"}},
			},
		},
		AssociatedBrowserScript: &agentstructs.BrowserScript{
			Author: "@Nariod",
			ScriptContents: `
function(task, responses) {
    let rows = [];
    for (let r = 0; r < responses.length; r++) {
        try {
            let data = JSON.parse(responses[r]);
            if (data.file_browser) {
                let fb = data.file_browser;
                if (fb.files) {
                    for (let f of fb.files) {
                        rows.push({
                            "Name": f.name,
                            "Size": f.is_file ? f.size : "",
                            "Permissions": f.permissions && f.permissions.Value ? f.permissions.Value : "",
                        });
                    }
                }
            }
        } catch(e) {}
    }
    return {"plaintext": task.user_output, "table": [{headers: [
        {"plaintext": "Name", "type": "string"},
        {"plaintext": "Size", "type": "number", "width": 100},
        {"plaintext": "Permissions", "type": "string", "width": 120},
    ], rows: rows}]};
}`,
		},
		TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
			if input == "" {
				input = "."
			}
			return args.SetArgValue("path", input)
		},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			resp := agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
			path, _ := taskData.Args.GetStringArg("path")
			resp.DisplayParams = &path
			return resp
		},
	})
}
