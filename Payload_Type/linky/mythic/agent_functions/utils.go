package agent_functions

import "strings"

// splitArgs splits a raw CLI string into at most n parts.
// The last part captures everything remaining (preserving spaces in paths).
func splitArgs(input string, n int) []string {
	input = strings.TrimSpace(input)
	if input == "" {
		return nil
	}
	parts := make([]string, 0, n)
	for i := 0; i < n-1; i++ {
		idx := strings.IndexByte(input, ' ')
		if idx < 0 {
			parts = append(parts, input)
			return parts
		}
		parts = append(parts, input[:idx])
		input = strings.TrimSpace(input[idx+1:])
	}
	if input != "" {
		parts = append(parts, input)
	}
	return parts
}
