// Copyright 2024-2026 Qualcomm Technologies, Inc. and/or its subsidiaries.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package common

import (
	"errors"
	"fmt"
	"net/url"
	"strings"

	"github.com/qcom-it-nexa-ai/geniex/cli/internal/model_hub"
	"github.com/qcom-it-nexa-ai/geniex/cli/internal/render"
)

// PrintError writes err to stdout in the Error theme, prefixed with "✘ ".
// Known structured errors get a multi-line, user-friendly rendering;
// unknown errors fall back to a single line.
//
// Caller still needs to `return err` after calling — this helper only prints.
func PrintError(err error) {
	if err == nil {
		return
	}

	var hub *model_hub.HubUnreachableError
	if errors.As(err, &hub) {
		printHubUnreachable(hub)
		return
	}

	fmt.Println(render.GetTheme().Error.Sprintf("✘ %s", err))
}

// PrintErrorf is sugar for PrintError(fmt.Errorf(format, args...)).
func PrintErrorf(format string, args ...any) {
	PrintError(fmt.Errorf(format, args...))
}

func printHubUnreachable(e *model_hub.HubUnreachableError) {
	host := e.URL
	if u, err := url.Parse(e.URL); err == nil && u.Host != "" {
		host = u.Host
	}

	var b strings.Builder
	fmt.Fprintf(&b, "✘ Unable to reach %s while resolving model metadata.\n", e.Hub)
	fmt.Fprintf(&b, "  Underlying: %v\n", e.Err)
	b.WriteString("  Possible causes: network timeout / corporate proxy / firewall\n")
	b.WriteString("  Try:\n")
	fmt.Fprintf(&b, "    - check browser access to %s\n", host)
	b.WriteString("    - configure proxy/network settings (HTTPS_PROXY)\n")
	b.WriteString("    - use --model-hub localfs --local-path <dir> for an already-downloaded model")

	fmt.Println(render.GetTheme().Error.Sprint(b.String()))
}
