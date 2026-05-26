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

package model_hub

import "fmt"

// HubUnreachableError signals that a hub HTTP endpoint could not be reached
// (DNS / dial / TLS / read timeout / context deadline). It is NOT used for
// auth or 4xx/5xx responses — those still flow through code2error or
// per-hub error paths.
type HubUnreachableError struct {
	Hub string // human label, e.g. "Hugging Face", "Qualcomm AI Hub"
	URL string // request URL that failed
	Err error  // underlying transport error
}

func (e *HubUnreachableError) Error() string {
	return fmt.Sprintf("%s unreachable: %s: %v", e.Hub, e.URL, e.Err)
}

func (e *HubUnreachableError) Unwrap() error { return e.Err }
