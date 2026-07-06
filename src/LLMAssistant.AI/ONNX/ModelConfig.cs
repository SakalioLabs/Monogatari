namespace LLMAssistant.AI.ONNX;

public class ModelConfig
{
    public string ModelPath { get; set; } = "";
    public string TokenizerPath { get; set; } = "";
    public int MaxSequenceLength { get; set; } = 2048;
    public int VocabSize { get; set; } = 32000;
    public bool UseDirectML { get; set; } = true;
    public int DeviceId { get; set; } = 0;
    public string[] ExecutionProviders { get; set; } = ["DML", "CPU"];
}
