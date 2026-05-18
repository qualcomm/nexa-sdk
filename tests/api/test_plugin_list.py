# Copyright 2024-2026 Qualcomm Technologies, Inc. and/or its subsidiaries.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""``geniex.get_plugin_list`` shape and contents."""

from __future__ import annotations

import geniex


def test_plugin_list_is_non_empty_string_list(geniex_session):
    plugins = geniex.get_plugin_list()
    assert isinstance(plugins, list) and plugins
    for p in plugins:
        assert isinstance(p, str) and p


def test_plugin_list_contains_llama_cpp(geniex_session):
    assert 'llama_cpp' in geniex.get_plugin_list()
