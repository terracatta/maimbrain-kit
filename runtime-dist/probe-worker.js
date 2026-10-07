// src/probe-worker.ts
self.onmessage = async (e) => {
  const report = { webgpu: "gpu" in navigator };
  try {
    const gpu = navigator.gpu;
    const adapter = gpu ? await gpu.requestAdapter() : null;
    report.adapter = adapter !== null;
    const ctx = e.data.canvas.getContext("webgpu");
    report.offscreenWebgpuContext = ctx !== null;
  } catch (err) {
    report.error = String(err);
  }
  self.postMessage(report);
};
