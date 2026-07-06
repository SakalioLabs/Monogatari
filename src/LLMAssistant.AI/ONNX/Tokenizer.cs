using System.Text;

namespace LLMAssistant.AI.ONNX;

public class Tokenizer
{
    private Dictionary<string, int> _vocab = new();
    private Dictionary<int, string> _reverseVocab = new();

    public int VocabSize => _vocab.Count;

    public void LoadFromJson(string path)
    {
        if (!File.Exists(path)) return;
        var json = File.ReadAllText(path);
        var vocab = System.Text.Json.JsonSerializer.Deserialize<Dictionary<string, int>>(json);
        if (vocab != null)
        {
            _vocab = vocab;
            _reverseVocab = vocab.ToDictionary(kv => kv.Value, kv => kv.Key);
        }
    }

    public int[] Encode(string text)
    {
        var tokens = new List<int>();
        var chars = text.ToCharArray();
        var current = new StringBuilder();

        foreach (var c in chars)
        {
            current.Append(c);
            var piece = current.ToString();
            if (_vocab.TryGetValue(piece, out var id))
            {
                tokens.Add(id);
                current.Clear();
            }
            else if (current.Length > 1)
            {
                var prev = current.ToString(0, current.Length - 1);
                if (_vocab.TryGetValue(prev, out var prevId))
                {
                    tokens.Add(prevId);
                    current.Clear();
                    current.Append(c);
                }
            }
        }

        if (current.Length > 0 && _vocab.TryGetValue(current.ToString(), out var lastId))
        {
            tokens.Add(lastId);
        }

        return tokens.ToArray();
    }

    public string Decode(int[] tokens)
    {
        var sb = new StringBuilder();
        foreach (var token in tokens)
        {
            if (_reverseVocab.TryGetValue(token, out var piece))
            {
                sb.Append(piece);
            }
        }
        return sb.ToString();
    }

    public string DecodeToken(int token)
    {
        return _reverseVocab.TryGetValue(token, out var piece) ? piece : "";
    }
}
