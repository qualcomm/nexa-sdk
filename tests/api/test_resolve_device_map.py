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

"""``geniex.resolve_device_map`` aliases — mirrors ``geniex_resolve_device``.

Source of truth lives in ``sdk/src/device.cpp``. Any change to the alias
table there must update these tests in the same PR.
"""

from __future__ import annotations

import geniex


def test_auto_resolves_to_a_known_plugin(geniex_session):
    plugin_id, device_id, ngl = geniex.resolve_device_map('auto')
    assert plugin_id in geniex.get_plugin_list()
    assert device_id is None or isinstance(device_id, str)
    assert ngl is None or isinstance(ngl, int)


def test_cpu_alias_zeroes_gpu_layers(geniex_session):
    plugin_id, _, ngl = geniex.resolve_device_map('cpu')
    assert plugin_id == 'llama_cpp'
    assert ngl == 0


def test_hybrid_alias_pins_gpu_layers_to_999(geniex_session):
    plugin_id, _, ngl = geniex.resolve_device_map('hybrid')
    assert plugin_id == 'llama_cpp'
    assert ngl == 999


def test_llama_cpp_npu_alias_pins_htp0(geniex_session):
    plugin_id, device_id, _ = geniex.resolve_device_map('llama_cpp:npu')
    assert plugin_id == 'llama_cpp'
    assert device_id == 'HTP0'


def test_qairt_npu_alias_resolves_to_qairt(geniex_session):
    plugin_id, device_id, _ = geniex.resolve_device_map('qairt:npu')
    assert plugin_id == 'qairt'
    assert isinstance(device_id, str) and device_id
