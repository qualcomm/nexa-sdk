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
	"log/slog"
	"os"

	geniex_sdk "github.com/qcom-it-nexa-ai/geniex/bindings/go"
	"github.com/qcom-it-nexa-ai/geniex/cli/internal/render"
)

// errorHints maps known sentinel errors to a multi-line hint shown after
// the headline error line. Keep the list short; only well-understood, user-
// actionable cases belong here.
var errorHints = []struct {
	is   error
	hint string
}{
	{geniex_sdk.ErrCommonParamNotSupported, `⚠️ A flag you passed is not supported by the plugin.

👉 Run 'geniex infer -h' to see which flags are plugin-specific.`},
	{geniex_sdk.ErrCommonNotSupport, `⚠️ Oops. This model type is not supported yet.

👉 Try these:
- Check back later for updates.
- See help in our discord or slack.`},
	{geniex_sdk.ErrCommonModelLoad, `⚠️ Oops. Model failed to load.

👉 Try these:
- Redownload the model.
- Verify your system meets the model's requirements.
- Check your NPU / GPU driver version and update it if it's out of date.
- See help in our discord or slack.`},
	{geniex_sdk.ErrCommonPluginLoad, `⚠️ Oops. Plugin failed to load.

👉 Try these:
- Ensure all plugin dependencies are correct.
- See help in our discord or slack.`},
	{geniex_sdk.ErrCommonPluginInvalid, `⚠️ Oops. Plugin is invalid.

👉 Try these:
- This model may not be compatible with your system. Try another model.
- See help in our discord or slack.`},
	{geniex_sdk.ErrLlmTokenizationContextLength, `Context length exceeded, please start a new conversation.`},
}

// PrintError renders err for the user on stderr in the theme's error style
// and logs the raw wrapped error via slog so the cause chain remains visible
// in the log stream. If err matches a known sentinel, the friendly hint is
// shown; otherwise the raw error message is shown.
func PrintError(err error) {
	if err == nil {
		return
	}
	slog.Error("cli error", "err", err)
	theme := render.GetTheme()
	for _, h := range errorHints {
		if errors.Is(err, h.is) {
			fmt.Fprintln(os.Stderr, theme.Error.Sprint(h.hint))
			return
		}
	}
	fmt.Fprintln(os.Stderr, theme.Error.Sprintf("Error: %s", err))
}
