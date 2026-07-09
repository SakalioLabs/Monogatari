using System.Text.Json;
using System.Text.Json.Nodes;
using LLMAssistant.Core;

namespace LLMAssistant.Game.Dialogue;

public static class DialogueParser
{
    public static DialogueScript? ParseFromJson(string json)
    {
        var root = JsonNode.Parse(json) as JsonObject
            ?? throw new JsonException("Dialogue JSON must contain an object.");

        NormalizeAlias(root, "start_node_id", "startNodeId");
        NormalizeNodes(root);

        return root.Deserialize<DialogueScript>(JsonOptions.Default);
    }

    public static async Task<DialogueScript?> LoadFromFile(string filePath)
    {
        if (!File.Exists(filePath)) return null;
        var json = await File.ReadAllTextAsync(filePath);
        return ParseFromJson(json);
    }

    public static string ToJson(DialogueScript script)
    {
        return JsonSerializer.Serialize(script, new JsonSerializerOptions
        {
            WriteIndented = true
        });
    }

    private static void NormalizeNodes(JsonObject root)
    {
        if (root["nodes"] is JsonObject nodeMap)
        {
            var nodes = new JsonArray();
            foreach (var (nodeId, value) in nodeMap)
            {
                if (value is not JsonObject sourceNode)
                    throw new JsonException($"Dialogue node '{nodeId}' must contain an object.");

                var node = (JsonObject)sourceNode.DeepClone();
                node["id"] = nodeId;
                NormalizeNode(node);
                nodes.Add(node);
            }

            root["nodes"] = nodes;
            return;
        }

        if (root["nodes"] is JsonArray nodeList)
        {
            foreach (var value in nodeList)
            {
                if (value is JsonObject node)
                    NormalizeNode(node);
            }
        }
    }

    private static void NormalizeNode(JsonObject node)
    {
        NormalizeAlias(node, "speaker_id", "speakerId");
        NormalizeAlias(node, "next_node_id", "nextNodeId");
        NormalizeAlias(node, "use_llm", "useLLM");
        NormalizeAlias(node, "llm_prompt", "llmPrompt");
        NormalizeAlias(node, "llm_system_prompt", "llmSystemPrompt");

        if (node["choices"] is not JsonArray choices) return;

        foreach (var value in choices)
        {
            if (value is not JsonObject choice) continue;
            NormalizeAlias(choice, "next_node_id", "nextNodeId");
            NormalizeAlias(choice, "relationship_changes", "relationshipChanges");
        }
    }

    private static void NormalizeAlias(JsonObject value, string alias, string propertyName)
    {
        if (!value.TryGetPropertyValue(alias, out var aliasValue)) return;

        value.Remove(alias);
        if (!value.ContainsKey(propertyName))
            value[propertyName] = aliasValue;
    }
}
