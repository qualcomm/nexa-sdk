#pragma once

#include <functional>
#include <string>
#include <unordered_map>

#include "pipeline/llm_pipeline.h"
#include "types.h"

// Model headers
#include "llama3_2.h"
#include "phi3_5.h"
#include "qwen3.h"

namespace geniex {

struct LlmModelEntry {
    std::function<std::optional<LLMPipeline>(const QnnRuntimeConfig&, const ModelConfig&)> make_pipeline;
};

inline const std::unordered_map<std::string, LlmModelEntry>& llm_model_registry() {
    static const std::unordered_map<std::string, LlmModelEntry> registry = {
        {"qwen3-4b", {qwen3_4b_instruct_2507::makePipeline}},
        {"qwen3-4b-base", {qwen3_4b::makePipeline}},
        {"qwen3-8b", {qwen3_8b::makePipeline}},
        {"phi3.5", {phi3_5::makePipeline}},
        {"llama3_2-1b", {llama3_2_1b::makePipeline}},
    };
    return registry;
}

}  // namespace geniex