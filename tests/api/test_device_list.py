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

"""``geniex.get_device_list`` shape per plugin."""

from __future__ import annotations

import geniex


def test_device_list_shape_for_each_plugin(geniex_session):
    for plugin in geniex.get_plugin_list():
        devices = geniex.get_device_list(plugin)
        assert isinstance(devices, list)
        for entry in devices:
            assert isinstance(entry, tuple) and len(entry) == 2
            device_id, label = entry
            assert isinstance(device_id, str) and device_id
            assert isinstance(label, str)
