using System.Diagnostics;

namespace LLMAssistant.Core.Time;

public class GameClock
{
    private readonly Stopwatch _stopwatch = new();
    private double _lastTime;
    private double _accumulator;

    public double DeltaTime { get; private set; }
    public double TotalTime { get; private set; }
    public double FixedDeltaTime { get; set; } = 1.0 / 60.0;
    public int FrameCount { get; private set; }
    public double FPS { get; private set; }

    private double _fpsAccumulator;
    private int _fpsFrameCount;

    public void Start()
    {
        _stopwatch.Start();
        _lastTime = _stopwatch.Elapsed.TotalSeconds;
    }

    public void Tick()
    {
        var currentTime = _stopwatch.Elapsed.TotalSeconds;
        DeltaTime = currentTime - _lastTime;
        _lastTime = currentTime;
        TotalTime = currentTime;
        FrameCount++;

        _fpsAccumulator += DeltaTime;
        _fpsFrameCount++;
        if (_fpsAccumulator >= 1.0)
        {
            FPS = _fpsFrameCount / _fpsAccumulator;
            _fpsAccumulator = 0;
            _fpsFrameCount = 0;
        }
    }

    public bool ShouldFixedUpdate()
    {
        _accumulator += DeltaTime;
        if (_accumulator >= FixedDeltaTime)
        {
            _accumulator -= FixedDeltaTime;
            return true;
        }
        return false;
    }

    public void Reset()
    {
        _stopwatch.Restart();
        _lastTime = 0;
        TotalTime = 0;
        DeltaTime = 0;
        FrameCount = 0;
        _accumulator = 0;
    }
}
