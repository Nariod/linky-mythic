package agent_functions

import agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"

func registerPs() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "ps", Description: "List running processes (populates Mythic process browser)", HelpString: "ps", Version: 1,
		MitreAttackMappings: []string{"T1057"},
		CommandAttributes:   agentstructs.CommandAttribute{SupportedOS: []string{agentstructs.SUPPORTED_OS_LINUX, agentstructs.SUPPORTED_OS_WINDOWS, agentstructs.SUPPORTED_OS_MACOS}},
		AssociatedBrowserScript: &agentstructs.BrowserScript{
			Author: "@Nariod",
			ScriptContents: `
function(task, responses) {
    let rows = [];
    for (let r = 0; r < responses.length; r++) {
        try {
            let data = JSON.parse(responses[r]);
            if (data.processes) {
                for (let p of data.processes) {
                    rows.push({
                        "PID": p.process_id,
                        "PPID": p.parent_process_id,
                        "Name": p.name,
                        "User": p.user || "",
                        "Cmd": p.command_line || "",
                        "Bin": p.bin_path || "",
                    });
                }
            }
        } catch(e) {}
    }
    return {"plaintext": task.user_output, "table": [{headers: [
        {"plaintext": "PID", "type": "number", "width": 80},
        {"plaintext": "PPID", "type": "number", "width": 80},
        {"plaintext": "Name", "type": "string"},
        {"plaintext": "User", "type": "string"},
        {"plaintext": "Cmd", "type": "string"},
        {"plaintext": "Bin", "type": "string"},
    ], rows: rows}]};
}`,
		},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			return agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
		},
	})
}
