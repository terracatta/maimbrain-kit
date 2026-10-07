// pkg/mb_host.js
function host_begin_frame() {
  wasm.host_begin_frame();
}
function host_draw_hash() {
  let deferred1_0;
  let deferred1_1;
  try {
    const ret = wasm.host_draw_hash();
    deferred1_0 = ret[0];
    deferred1_1 = ret[1];
    return getStringFromWasm0(ret[0], ret[1]);
  } finally {
    wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
  }
}
function host_end_frame() {
  const ret = wasm.host_end_frame();
  if (ret[1]) {
    throw takeFromExternrefTable0(ret[0]);
  }
}
function host_info() {
  let deferred1_0;
  let deferred1_1;
  try {
    const ret = wasm.host_info();
    deferred1_0 = ret[0];
    deferred1_1 = ret[1];
    return getStringFromWasm0(ret[0], ret[1]);
  } finally {
    wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
  }
}
function host_init(canvas, logical_w, logical_h) {
  const ret = wasm.host_init(canvas, logical_w, logical_h);
  return ret;
}
function host_new_session() {
  wasm.host_new_session();
}
function host_resize(width, height) {
  wasm.host_resize(width, height);
}
function __wbg_get_imports() {
  const import0 = {
    __proto__: null,
    __wbg_Window_a2a6c4d665047b14: function(arg0) {
      const ret = arg0.Window;
      return ret;
    },
    __wbg_WorkerGlobalScope_2664448a7c667d67: function(arg0) {
      const ret = arg0.WorkerGlobalScope;
      return ret;
    },
    __wbg___wbindgen_debug_string_4687d8d8c2017d52: function(arg0, arg1) {
      const ret = debugString(arg1);
      const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len1 = WASM_VECTOR_LEN;
      getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
      getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    },
    __wbg___wbindgen_is_function_1f9d30630b8b1d3d: function(arg0) {
      const ret = typeof arg0 === "function";
      return ret;
    },
    __wbg___wbindgen_is_null_e343b7d08827ba72: function(arg0) {
      const ret = arg0 === null;
      return ret;
    },
    __wbg___wbindgen_is_string_90b56bc79aad6f6c: function(arg0) {
      const ret = typeof arg0 === "string";
      return ret;
    },
    __wbg___wbindgen_is_undefined_8865fb403f8fe9d8: function(arg0) {
      const ret = arg0 === void 0;
      return ret;
    },
    __wbg___wbindgen_throw_41e9ee4f547fc59a: function(arg0, arg1) {
      throw new Error(getStringFromWasm0(arg0, arg1));
    },
    __wbg__wbg_cb_unref_dcc1a90847f04c41: function(arg0) {
      arg0._wbg_cb_unref();
    },
    __wbg_beginRenderPass_3c53642423af50dc: function() {
      return handleError(function(arg0, arg1) {
        const ret = arg0.beginRenderPass(arg1);
        return ret;
      }, arguments);
    },
    __wbg_call_187d372bd5fdd4aa: function() {
      return handleError(function(arg0, arg1, arg2) {
        const ret = arg0.call(arg1, arg2);
        return ret;
      }, arguments);
    },
    __wbg_configure_1e2c1c9edad07d26: function() {
      return handleError(function(arg0, arg1) {
        arg0.configure(arg1);
      }, arguments);
    },
    __wbg_createBindGroupLayout_b1bd63b4e88459d8: function() {
      return handleError(function(arg0, arg1) {
        const ret = arg0.createBindGroupLayout(arg1);
        return ret;
      }, arguments);
    },
    __wbg_createBindGroup_f539b26ca341308f: function(arg0, arg1) {
      const ret = arg0.createBindGroup(arg1);
      return ret;
    },
    __wbg_createBuffer_d800e9b1d41b2ee5: function() {
      return handleError(function(arg0, arg1) {
        const ret = arg0.createBuffer(arg1);
        return ret;
      }, arguments);
    },
    __wbg_createCommandEncoder_3352d1ffc36c6fc0: function(arg0, arg1) {
      const ret = arg0.createCommandEncoder(arg1);
      return ret;
    },
    __wbg_createPipelineLayout_6eab52c327118937: function(arg0, arg1) {
      const ret = arg0.createPipelineLayout(arg1);
      return ret;
    },
    __wbg_createRenderPipeline_0ebb7ebc653e9207: function() {
      return handleError(function(arg0, arg1) {
        const ret = arg0.createRenderPipeline(arg1);
        return ret;
      }, arguments);
    },
    __wbg_createSampler_9bd91d7e928c0060: function(arg0, arg1) {
      const ret = arg0.createSampler(arg1);
      return ret;
    },
    __wbg_createShaderModule_cefa51336cb288ae: function(arg0, arg1) {
      const ret = arg0.createShaderModule(arg1);
      return ret;
    },
    __wbg_createTexture_ed7e9fc04dd54d84: function() {
      return handleError(function(arg0, arg1) {
        const ret = arg0.createTexture(arg1);
        return ret;
      }, arguments);
    },
    __wbg_createView_da41c2d2cb212715: function() {
      return handleError(function(arg0, arg1) {
        const ret = arg0.createView(arg1);
        return ret;
      }, arguments);
    },
    __wbg_description_83b8a393160021b9: function(arg0, arg1) {
      const ret = arg1.description;
      const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len1 = WASM_VECTOR_LEN;
      getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
      getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    },
    __wbg_document_9854e03c05fc8834: function(arg0) {
      const ret = arg0.document;
      return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    },
    __wbg_draw_086a9578fc9898c2: function(arg0, arg1, arg2, arg3, arg4) {
      arg0.draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
    },
    __wbg_end_b57473834b877409: function(arg0) {
      arg0.end();
    },
    __wbg_finish_09ec094c10f41e7b: function(arg0) {
      const ret = arg0.finish();
      return ret;
    },
    __wbg_finish_ec1c191f66a895b1: function(arg0, arg1) {
      const ret = arg0.finish(arg1);
      return ret;
    },
    __wbg_getContext_635e36719cad2623: function() {
      return handleError(function(arg0, arg1, arg2) {
        const ret = arg0.getContext(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
      }, arguments);
    },
    __wbg_getContext_e567868594d2e0c6: function() {
      return handleError(function(arg0, arg1, arg2) {
        const ret = arg0.getContext(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
      }, arguments);
    },
    __wbg_getCurrentTexture_9f3b84d0eaa6cd95: function() {
      return handleError(function(arg0) {
        const ret = arg0.getCurrentTexture();
        return ret;
      }, arguments);
    },
    __wbg_getPreferredCanvasFormat_0ef5034c8902201b: function(arg0) {
      const ret = arg0.getPreferredCanvasFormat();
      return (__wbindgen_enum_GpuTextureFormat.indexOf(ret) + 1 || 102) - 1;
    },
    __wbg_get_fd12a9100976c296: function(arg0, arg1) {
      const ret = arg0[arg1 >>> 0];
      return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    },
    __wbg_gpu_afdd4387c7afe5f9: function(arg0) {
      const ret = arg0.gpu;
      return ret;
    },
    __wbg_height_aa756cf29b6d629c: function(arg0) {
      const ret = arg0.height;
      return ret;
    },
    __wbg_height_fc97e1a0c2e7331f: function(arg0) {
      const ret = arg0.height;
      return ret;
    },
    __wbg_info_971d8b9db3dae69f: function(arg0) {
      const ret = arg0.info;
      return ret;
    },
    __wbg_instanceof_Window_82d71df4eddf88bc: function(arg0) {
      let result;
      try {
        result = arg0 instanceof Window;
      } catch (_) {
        result = false;
      }
      const ret = result;
      return ret;
    },
    __wbg_isFallbackAdapter_4c8cc3b18677460a: function(arg0) {
      const ret = arg0.isFallbackAdapter;
      return ret;
    },
    __wbg_label_7add8cb37a6ef98f: function(arg0, arg1) {
      const ret = arg1.label;
      const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
      const len1 = WASM_VECTOR_LEN;
      getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
      getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    },
    __wbg_limits_601ad2e086ef8141: function(arg0) {
      const ret = arg0.limits;
      return ret;
    },
    __wbg_mapAsync_b0597127f5037286: function(arg0, arg1, arg2, arg3) {
      const ret = arg0.mapAsync(arg1 >>> 0, arg2, arg3);
      return ret;
    },
    __wbg_maxBindGroupsPlusVertexBuffers_52369f089736ef9d: function(arg0) {
      const ret = arg0.maxBindGroupsPlusVertexBuffers;
      return ret;
    },
    __wbg_maxBindGroups_4e424afe6ce86ca2: function(arg0) {
      const ret = arg0.maxBindGroups;
      return ret;
    },
    __wbg_maxBindingsPerBindGroup_7d035da36821c44f: function(arg0) {
      const ret = arg0.maxBindingsPerBindGroup;
      return ret;
    },
    __wbg_maxBufferSize_423f4a084e32a195: function(arg0) {
      const ret = arg0.maxBufferSize;
      return ret;
    },
    __wbg_maxColorAttachmentBytesPerSample_c4cd9126f6d287c6: function(arg0) {
      const ret = arg0.maxColorAttachmentBytesPerSample;
      return ret;
    },
    __wbg_maxColorAttachments_d924670762b9e250: function(arg0) {
      const ret = arg0.maxColorAttachments;
      return ret;
    },
    __wbg_maxComputeInvocationsPerWorkgroup_707a3868f7cebb59: function(arg0) {
      const ret = arg0.maxComputeInvocationsPerWorkgroup;
      return ret;
    },
    __wbg_maxComputeWorkgroupSizeX_0a4d99463cbd6e5e: function(arg0) {
      const ret = arg0.maxComputeWorkgroupSizeX;
      return ret;
    },
    __wbg_maxComputeWorkgroupSizeY_85123ea0587f7558: function(arg0) {
      const ret = arg0.maxComputeWorkgroupSizeY;
      return ret;
    },
    __wbg_maxComputeWorkgroupSizeZ_a3186b4c5267d44f: function(arg0) {
      const ret = arg0.maxComputeWorkgroupSizeZ;
      return ret;
    },
    __wbg_maxComputeWorkgroupStorageSize_57b297355cfb6204: function(arg0) {
      const ret = arg0.maxComputeWorkgroupStorageSize;
      return ret;
    },
    __wbg_maxComputeWorkgroupsPerDimension_4158f95e673d54c4: function(arg0) {
      const ret = arg0.maxComputeWorkgroupsPerDimension;
      return ret;
    },
    __wbg_maxDynamicStorageBuffersPerPipelineLayout_226b0b70910aa16c: function(arg0) {
      const ret = arg0.maxDynamicStorageBuffersPerPipelineLayout;
      return ret;
    },
    __wbg_maxDynamicUniformBuffersPerPipelineLayout_0e835fda711fc7e6: function(arg0) {
      const ret = arg0.maxDynamicUniformBuffersPerPipelineLayout;
      return ret;
    },
    __wbg_maxInterStageShaderVariables_8c4a1d727e2aa35a: function(arg0) {
      const ret = arg0.maxInterStageShaderVariables;
      return ret;
    },
    __wbg_maxSampledTexturesPerShaderStage_6675f5e91d9a728a: function(arg0) {
      const ret = arg0.maxSampledTexturesPerShaderStage;
      return ret;
    },
    __wbg_maxSamplersPerShaderStage_1910fa38a6ed1e1f: function(arg0) {
      const ret = arg0.maxSamplersPerShaderStage;
      return ret;
    },
    __wbg_maxStorageBufferBindingSize_2e244bded070b18d: function(arg0) {
      const ret = arg0.maxStorageBufferBindingSize;
      return ret;
    },
    __wbg_maxStorageBuffersPerShaderStage_a285f3ebca51ca0d: function(arg0) {
      const ret = arg0.maxStorageBuffersPerShaderStage;
      return ret;
    },
    __wbg_maxStorageTexturesPerShaderStage_7aa946f0fc322a2b: function(arg0) {
      const ret = arg0.maxStorageTexturesPerShaderStage;
      return ret;
    },
    __wbg_maxTextureArrayLayers_0e699147ad00502d: function(arg0) {
      const ret = arg0.maxTextureArrayLayers;
      return ret;
    },
    __wbg_maxTextureDimension1D_aabf6add54decfe2: function(arg0) {
      const ret = arg0.maxTextureDimension1D;
      return ret;
    },
    __wbg_maxTextureDimension2D_dd598b27e9c0c1c4: function(arg0) {
      const ret = arg0.maxTextureDimension2D;
      return ret;
    },
    __wbg_maxTextureDimension3D_f944266c65dfd1a9: function(arg0) {
      const ret = arg0.maxTextureDimension3D;
      return ret;
    },
    __wbg_maxUniformBufferBindingSize_59fa6be7cfbeeb53: function(arg0) {
      const ret = arg0.maxUniformBufferBindingSize;
      return ret;
    },
    __wbg_maxUniformBuffersPerShaderStage_bee5f00a4d706c7f: function(arg0) {
      const ret = arg0.maxUniformBuffersPerShaderStage;
      return ret;
    },
    __wbg_maxVertexAttributes_5cf6392c4e9033fe: function(arg0) {
      const ret = arg0.maxVertexAttributes;
      return ret;
    },
    __wbg_maxVertexBufferArrayStride_548baa887375d865: function(arg0) {
      const ret = arg0.maxVertexBufferArrayStride;
      return ret;
    },
    __wbg_maxVertexBuffers_75d881156591f5da: function(arg0) {
      const ret = arg0.maxVertexBuffers;
      return ret;
    },
    __wbg_minStorageBufferOffsetAlignment_5ba9b77792bdadb3: function(arg0) {
      const ret = arg0.minStorageBufferOffsetAlignment;
      return ret;
    },
    __wbg_minUniformBufferOffsetAlignment_ab7d52a5293b22bd: function(arg0) {
      const ret = arg0.minUniformBufferOffsetAlignment;
      return ret;
    },
    __wbg_navigator_2156486643462a87: function(arg0) {
      const ret = arg0.navigator;
      return ret;
    },
    __wbg_navigator_c9bce48d4b578c9c: function(arg0) {
      const ret = arg0.navigator;
      return ret;
    },
    __wbg_new_617a8cdb8bb1130e: function() {
      const ret = new Object();
      return ret;
    },
    __wbg_new_a217df4351db4b9b: function() {
      return handleError(function(arg0, arg1) {
        const ret = new OffscreenCanvas(arg0 >>> 0, arg1 >>> 0);
        return ret;
      }, arguments);
    },
    __wbg_new_typed_689a3a281a8d4da6: function() {
      const ret = new Object();
      return ret;
    },
    __wbg_new_typed_b01cb72a8af741a3: function(arg0, arg1) {
      try {
        var state0 = { a: arg0, b: arg1 };
        var cb0 = (arg02, arg12) => {
          const a = state0.a;
          state0.a = 0;
          try {
            return wasm_bindgen_16ce60f5be4e30c6___convert__closures_____invoke___js_sys_ad02f14a055e7bf2___Function_fn_wasm_bindgen_16ce60f5be4e30c6___JsValue_____wasm_bindgen_16ce60f5be4e30c6___sys__Undefined___js_sys_ad02f14a055e7bf2___Function_fn_wasm_bindgen_16ce60f5be4e30c6___JsValue_____wasm_bindgen_16ce60f5be4e30c6___sys__Undefined_______true_(a, state0.b, arg02, arg12);
          } finally {
            state0.a = a;
          }
        };
        const ret = new Promise(cb0);
        return ret;
      } finally {
        state0.a = 0;
      }
    },
    __wbg_onSubmittedWorkDone_1190213cee1ecf7e: function(arg0) {
      const ret = arg0.onSubmittedWorkDone();
      return ret;
    },
    __wbg_querySelectorAll_6aebfe3df1fb2014: function() {
      return handleError(function(arg0, arg1, arg2) {
        const ret = arg0.querySelectorAll(getStringFromWasm0(arg1, arg2));
        return ret;
      }, arguments);
    },
    __wbg_queueMicrotask_9833f9a49df95a49: function(arg0) {
      const ret = arg0.queueMicrotask;
      return ret;
    },
    __wbg_queueMicrotask_a72f977e97f23c5f: function(arg0) {
      queueMicrotask(arg0);
    },
    __wbg_queue_7b62c28143d44293: function(arg0) {
      const ret = arg0.queue;
      return ret;
    },
    __wbg_requestAdapter_a539af006419f2e9: function(arg0, arg1) {
      const ret = arg0.requestAdapter(arg1);
      return ret;
    },
    __wbg_requestDevice_5cb8a582e55d08cb: function(arg0, arg1) {
      const ret = arg0.requestDevice(arg1);
      return ret;
    },
    __wbg_resolve_0076e10020304ede: function(arg0) {
      const ret = Promise.resolve(arg0);
      return ret;
    },
    __wbg_setBindGroup_11bdbb60cc8b54b9: function() {
      return handleError(function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
        arg0.setBindGroup(arg1 >>> 0, arg2, getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
      }, arguments);
    },
    __wbg_setBindGroup_418c3e0eb6943ce0: function(arg0, arg1, arg2) {
      arg0.setBindGroup(arg1 >>> 0, arg2);
    },
    __wbg_setPipeline_b6f981027e02cd16: function(arg0, arg1) {
      arg0.setPipeline(arg1);
    },
    __wbg_setVertexBuffer_6db3b60e99280744: function(arg0, arg1, arg2, arg3) {
      arg0.setVertexBuffer(arg1 >>> 0, arg2, arg3);
    },
    __wbg_setVertexBuffer_cbf4ca1627c02f4c: function(arg0, arg1, arg2, arg3, arg4) {
      arg0.setVertexBuffer(arg1 >>> 0, arg2, arg3, arg4);
    },
    __wbg_setViewport_c0be08dcc8965ccf: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
      arg0.setViewport(arg1, arg2, arg3, arg4, arg5, arg6);
    },
    __wbg_set_145a351398b48c65: function() {
      return handleError(function(arg0, arg1, arg2) {
        const ret = Reflect.set(arg0, arg1, arg2);
        return ret;
      }, arguments);
    },
    __wbg_set_a_82818effc94f6256: function(arg0, arg1) {
      arg0.a = arg1;
    },
    __wbg_set_access_a099cfbbeec9b96f: function(arg0, arg1) {
      arg0.access = __wbindgen_enum_GpuStorageTextureAccess[arg1];
    },
    __wbg_set_address_mode_u_a68737cf5d288f95: function(arg0, arg1) {
      arg0.addressModeU = __wbindgen_enum_GpuAddressMode[arg1];
    },
    __wbg_set_address_mode_v_b1c3c45933f540d1: function(arg0, arg1) {
      arg0.addressModeV = __wbindgen_enum_GpuAddressMode[arg1];
    },
    __wbg_set_address_mode_w_889c31cf7022c764: function(arg0, arg1) {
      arg0.addressModeW = __wbindgen_enum_GpuAddressMode[arg1];
    },
    __wbg_set_alpha_106f21a936a85eba: function(arg0, arg1) {
      arg0.alpha = arg1;
    },
    __wbg_set_alpha_mode_5544568dbac50280: function(arg0, arg1) {
      arg0.alphaMode = __wbindgen_enum_GpuCanvasAlphaMode[arg1];
    },
    __wbg_set_alpha_to_coverage_enabled_3372ce329447b8f1: function(arg0, arg1) {
      arg0.alphaToCoverageEnabled = arg1 !== 0;
    },
    __wbg_set_array_layer_count_22afa0a979e4ad55: function(arg0, arg1) {
      arg0.arrayLayerCount = arg1 >>> 0;
    },
    __wbg_set_array_stride_f64_6816040e5e7598c3: function(arg0, arg1) {
      arg0.arrayStride = arg1;
    },
    __wbg_set_aspect_a48d046965270281: function(arg0, arg1) {
      arg0.aspect = __wbindgen_enum_GpuTextureAspect[arg1];
    },
    __wbg_set_aspect_b1a9909bf315433f: function(arg0, arg1) {
      arg0.aspect = __wbindgen_enum_GpuTextureAspect[arg1];
    },
    __wbg_set_attributes_9e38cb1dde387a5b: function(arg0, arg1, arg2) {
      arg0.attributes = getArrayJsValueViewFromWasm0(arg1, arg2);
    },
    __wbg_set_b_a3297ee7e7cac3a8: function(arg0, arg1) {
      arg0.b = arg1;
    },
    __wbg_set_base_array_layer_2435ba92c80346ae: function(arg0, arg1) {
      arg0.baseArrayLayer = arg1 >>> 0;
    },
    __wbg_set_base_mip_level_8b6093e875e7c65d: function(arg0, arg1) {
      arg0.baseMipLevel = arg1 >>> 0;
    },
    __wbg_set_beginning_of_pass_write_index_e552c5e8b8bbf52f: function(arg0, arg1) {
      arg0.beginningOfPassWriteIndex = arg1 >>> 0;
    },
    __wbg_set_bind_group_layouts_458c44ba55100b82: function(arg0, arg1, arg2) {
      arg0.bindGroupLayouts = getArrayJsValueViewFromWasm0(arg1, arg2);
    },
    __wbg_set_binding_81b3fac7f7acaf8d: function(arg0, arg1) {
      arg0.binding = arg1 >>> 0;
    },
    __wbg_set_binding_b6cee57f35ac5190: function(arg0, arg1) {
      arg0.binding = arg1 >>> 0;
    },
    __wbg_set_blend_1a801617945f7945: function(arg0, arg1) {
      arg0.blend = arg1;
    },
    __wbg_set_buffer_1548ae88a9188037: function(arg0, arg1) {
      arg0.buffer = arg1;
    },
    __wbg_set_buffer_8d0ac64ad20dfc84: function(arg0, arg1) {
      arg0.buffer = arg1;
    },
    __wbg_set_buffers_5d0e0c50791f710e: function(arg0, arg1, arg2) {
      arg0.buffers = getArrayJsValueViewFromWasm0(arg1, arg2);
    },
    __wbg_set_bytes_per_row_c28583f0063160f1: function(arg0, arg1) {
      arg0.bytesPerRow = arg1 >>> 0;
    },
    __wbg_set_clear_value_gpu_color_dict_a9f763e8372ac1de: function(arg0, arg1) {
      arg0.clearValue = arg1;
    },
    __wbg_set_code_5d5b0b9e2fd0dca7: function(arg0, arg1, arg2) {
      arg0.code = getStringFromWasm0(arg1, arg2);
    },
    __wbg_set_color_8ecace4011f47d2e: function(arg0, arg1) {
      arg0.color = arg1;
    },
    __wbg_set_color_attachments_622fe2d5997fda7a: function(arg0, arg1, arg2) {
      arg0.colorAttachments = getArrayJsValueViewFromWasm0(arg1, arg2);
    },
    __wbg_set_compare_080c9e492ff36990: function(arg0, arg1) {
      arg0.compare = __wbindgen_enum_GpuCompareFunction[arg1];
    },
    __wbg_set_compare_817cf3695599eaa6: function(arg0, arg1) {
      arg0.compare = __wbindgen_enum_GpuCompareFunction[arg1];
    },
    __wbg_set_count_8ff0c9474e39a849: function(arg0, arg1) {
      arg0.count = arg1 >>> 0;
    },
    __wbg_set_cull_mode_85d2b4ab0ce3a564: function(arg0, arg1) {
      arg0.cullMode = __wbindgen_enum_GpuCullMode[arg1];
    },
    __wbg_set_depth_bias_95abf479cae3f3cd: function(arg0, arg1) {
      arg0.depthBias = arg1;
    },
    __wbg_set_depth_bias_clamp_ba3d0b8348151350: function(arg0, arg1) {
      arg0.depthBiasClamp = arg1;
    },
    __wbg_set_depth_bias_slope_scale_6b2584d93f5b9cd2: function(arg0, arg1) {
      arg0.depthBiasSlopeScale = arg1;
    },
    __wbg_set_depth_clear_value_e30a4c754c6b3b26: function(arg0, arg1) {
      arg0.depthClearValue = arg1;
    },
    __wbg_set_depth_compare_a90de4e3714397ab: function(arg0, arg1) {
      arg0.depthCompare = __wbindgen_enum_GpuCompareFunction[arg1];
    },
    __wbg_set_depth_fail_op_b5c64541d1b6b482: function(arg0, arg1) {
      arg0.depthFailOp = __wbindgen_enum_GpuStencilOperation[arg1];
    },
    __wbg_set_depth_load_op_932888016d762d3e: function(arg0, arg1) {
      arg0.depthLoadOp = __wbindgen_enum_GpuLoadOp[arg1];
    },
    __wbg_set_depth_or_array_layers_e2f074a0284e4806: function(arg0, arg1) {
      arg0.depthOrArrayLayers = arg1 >>> 0;
    },
    __wbg_set_depth_read_only_be790175a1c2db9a: function(arg0, arg1) {
      arg0.depthReadOnly = arg1 !== 0;
    },
    __wbg_set_depth_stencil_attachment_54a8922f5fbe08bf: function(arg0, arg1) {
      arg0.depthStencilAttachment = arg1;
    },
    __wbg_set_depth_stencil_b7cffc59ad4da529: function(arg0, arg1) {
      arg0.depthStencil = arg1;
    },
    __wbg_set_depth_store_op_9054814f164ab55d: function(arg0, arg1) {
      arg0.depthStoreOp = __wbindgen_enum_GpuStoreOp[arg1];
    },
    __wbg_set_depth_write_enabled_31a821ee1fb3b0b3: function(arg0, arg1) {
      arg0.depthWriteEnabled = arg1 !== 0;
    },
    __wbg_set_device_210484a77b675c9c: function(arg0, arg1) {
      arg0.device = arg1;
    },
    __wbg_set_dimension_3da9d03131a9f446: function(arg0, arg1) {
      arg0.dimension = __wbindgen_enum_GpuTextureDimension[arg1];
    },
    __wbg_set_dimension_56332450afa3e0c0: function(arg0, arg1) {
      arg0.dimension = __wbindgen_enum_GpuTextureViewDimension[arg1];
    },
    __wbg_set_dst_factor_865ba9aaf187890c: function(arg0, arg1) {
      arg0.dstFactor = __wbindgen_enum_GpuBlendFactor[arg1];
    },
    __wbg_set_end_of_pass_write_index_8f164f9e60d4ad16: function(arg0, arg1) {
      arg0.endOfPassWriteIndex = arg1 >>> 0;
    },
    __wbg_set_entries_6f866302103b81e9: function(arg0, arg1, arg2) {
      arg0.entries = getArrayJsValueViewFromWasm0(arg1, arg2);
    },
    __wbg_set_entries_f26b77ab9548e906: function(arg0, arg1, arg2) {
      arg0.entries = getArrayJsValueViewFromWasm0(arg1, arg2);
    },
    __wbg_set_entry_point_71cef95c137b5774: function(arg0, arg1, arg2) {
      arg0.entryPoint = getStringFromWasm0(arg1, arg2);
    },
    __wbg_set_entry_point_b70f98f5025a114d: function(arg0, arg1, arg2) {
      arg0.entryPoint = getStringFromWasm0(arg1, arg2);
    },
    __wbg_set_external_texture_7f966c604c4f8098: function(arg0, arg1) {
      arg0.externalTexture = arg1;
    },
    __wbg_set_fail_op_d59d0187e4111dfe: function(arg0, arg1) {
      arg0.failOp = __wbindgen_enum_GpuStencilOperation[arg1];
    },
    __wbg_set_format_23f7f32549751d43: function(arg0, arg1) {
      arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
    },
    __wbg_set_format_283dca56552f07a3: function(arg0, arg1) {
      arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
    },
    __wbg_set_format_5080a858117ad2c1: function(arg0, arg1) {
      arg0.format = __wbindgen_enum_GpuVertexFormat[arg1];
    },
    __wbg_set_format_66735b94bd868ba2: function(arg0, arg1) {
      arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
    },
    __wbg_set_format_7f2bdbfb101b1ae1: function(arg0, arg1) {
      arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
    },
    __wbg_set_format_92732ea75d3b79f5: function(arg0, arg1) {
      arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
    },
    __wbg_set_format_f009e603f7d4c28e: function(arg0, arg1) {
      arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
    },
    __wbg_set_fragment_d2b0ec97d7cf8d47: function(arg0, arg1) {
      arg0.fragment = arg1;
    },
    __wbg_set_front_face_d3f8a2e07e7b25dd: function(arg0, arg1) {
      arg0.frontFace = __wbindgen_enum_GpuFrontFace[arg1];
    },
    __wbg_set_g_b527ee8a9bed553d: function(arg0, arg1) {
      arg0.g = arg1;
    },
    __wbg_set_has_dynamic_offset_0c72ffa900c5a269: function(arg0, arg1) {
      arg0.hasDynamicOffset = arg1 !== 0;
    },
    __wbg_set_height_c9789c1c77eaedff: function(arg0, arg1) {
      arg0.height = arg1 >>> 0;
    },
    __wbg_set_height_f6619158e5735877: function(arg0, arg1) {
      arg0.height = arg1 >>> 0;
    },
    __wbg_set_height_fde391767df1ce27: function(arg0, arg1) {
      arg0.height = arg1 >>> 0;
    },
    __wbg_set_label_17202740051e9722: function(arg0, arg1, arg2) {
      arg0.label = getStringFromWasm0(arg1, arg2);
    },
    __wbg_set_label_2fefb39c0e0dbbe8: function(arg0, arg1, arg2) {
      arg0.label = getStringFromWasm0(arg1, arg2);
    },
    __wbg_set_label_3cb2322e6f6db14c: function(arg0, arg1, arg2) {
      arg0.label = getStringFromWasm0(arg1, arg2);
    },
    __wbg_set_label_3f2ccaafef5ff7c9: function(arg0, arg1, arg2) {
      arg0.label = getStringFromWasm0(arg1, arg2);
    },
    __wbg_set_label_612add98a4398f92: function(arg0, arg1, arg2) {
      arg0.label = getStringFromWasm0(arg1, arg2);
    },
    __wbg_set_label_70a09ee68d6b1b26: function(arg0, arg1, arg2) {
      arg0.label = getStringFromWasm0(arg1, arg2);
    },
    __wbg_set_label_92cd3811e96b487c: function(arg0, arg1, arg2) {
      arg0.label = getStringFromWasm0(arg1, arg2);
    },
    __wbg_set_label_9c2a186152427ee0: function(arg0, arg1, arg2) {
      arg0.label = getStringFromWasm0(arg1, arg2);
    },
    __wbg_set_label_c3eaf136aa464cba: function(arg0, arg1, arg2) {
      arg0.label = getStringFromWasm0(arg1, arg2);
    },
    __wbg_set_label_c7987704d29f284b: function(arg0, arg1, arg2) {
      arg0.label = getStringFromWasm0(arg1, arg2);
    },
    __wbg_set_label_cfe64bca8945ee30: function(arg0, arg1, arg2) {
      arg0.label = getStringFromWasm0(arg1, arg2);
    },
    __wbg_set_label_e02179cf97e95763: function(arg0, arg1, arg2) {
      arg0.label = getStringFromWasm0(arg1, arg2);
    },
    __wbg_set_label_ee172cd5f6a96961: function(arg0, arg1, arg2) {
      arg0.label = getStringFromWasm0(arg1, arg2);
    },
    __wbg_set_layout_454e3a091b390cd4: function(arg0, arg1) {
      arg0.layout = arg1;
    },
    __wbg_set_layout_75dc1ca3f2421cff: function(arg0, arg1) {
      arg0.layout = arg1;
    },
    __wbg_set_layout_gpu_auto_layout_mode_06a2b95af1043098: function(arg0, arg1) {
      arg0.layout = __wbindgen_enum_GpuAutoLayoutMode[arg1];
    },
    __wbg_set_load_op_c56b1269acc2d51f: function(arg0, arg1) {
      arg0.loadOp = __wbindgen_enum_GpuLoadOp[arg1];
    },
    __wbg_set_lod_max_clamp_db24179f67f3aa31: function(arg0, arg1) {
      arg0.lodMaxClamp = arg1;
    },
    __wbg_set_lod_min_clamp_2bbce566e9fefa04: function(arg0, arg1) {
      arg0.lodMinClamp = arg1;
    },
    __wbg_set_mag_filter_db8e6b42d4f8846d: function(arg0, arg1) {
      arg0.magFilter = __wbindgen_enum_GpuFilterMode[arg1];
    },
    __wbg_set_mapped_at_creation_3f320fef6761b02c: function(arg0, arg1) {
      arg0.mappedAtCreation = arg1 !== 0;
    },
    __wbg_set_mask_c1079e551ec360dc: function(arg0, arg1) {
      arg0.mask = arg1 >>> 0;
    },
    __wbg_set_max_anisotropy_84749fdcec362dc4: function(arg0, arg1) {
      arg0.maxAnisotropy = arg1;
    },
    __wbg_set_min_binding_size_f64_897e3cd4496ddec9: function(arg0, arg1) {
      arg0.minBindingSize = arg1;
    },
    __wbg_set_min_filter_d435bbfc5a637757: function(arg0, arg1) {
      arg0.minFilter = __wbindgen_enum_GpuFilterMode[arg1];
    },
    __wbg_set_mip_level_count_047936c630acee7b: function(arg0, arg1) {
      arg0.mipLevelCount = arg1 >>> 0;
    },
    __wbg_set_mip_level_count_44bc46a1ae6f6daa: function(arg0, arg1) {
      arg0.mipLevelCount = arg1 >>> 0;
    },
    __wbg_set_mip_level_f3745730372683d5: function(arg0, arg1) {
      arg0.mipLevel = arg1 >>> 0;
    },
    __wbg_set_mipmap_filter_62fb49a84b0747ff: function(arg0, arg1) {
      arg0.mipmapFilter = __wbindgen_enum_GpuMipmapFilterMode[arg1];
    },
    __wbg_set_mode_7edfbc344ef9c650: function(arg0, arg1) {
      arg0.mode = __wbindgen_enum_GpuCanvasToneMappingMode[arg1];
    },
    __wbg_set_module_392eeaa269f203b0: function(arg0, arg1) {
      arg0.module = arg1;
    },
    __wbg_set_module_715d37652c4998ec: function(arg0, arg1) {
      arg0.module = arg1;
    },
    __wbg_set_multisample_ff72a7a5456cbeb7: function(arg0, arg1) {
      arg0.multisample = arg1;
    },
    __wbg_set_multisampled_039f032dc4b67367: function(arg0, arg1) {
      arg0.multisampled = arg1 !== 0;
    },
    __wbg_set_offset_f64_127e8a0aa5c5485a: function(arg0, arg1) {
      arg0.offset = arg1;
    },
    __wbg_set_offset_f64_457756429ede426d: function(arg0, arg1) {
      arg0.offset = arg1;
    },
    __wbg_set_offset_f64_a903425d5a8e5815: function(arg0, arg1) {
      arg0.offset = arg1;
    },
    __wbg_set_operation_00a77386523b88f9: function(arg0, arg1) {
      arg0.operation = __wbindgen_enum_GpuBlendOperation[arg1];
    },
    __wbg_set_origin_gpu_origin_3d_dict_0619d4860adb4eb6: function(arg0, arg1) {
      arg0.origin = arg1;
    },
    __wbg_set_pass_op_3cf10feb3d76ab97: function(arg0, arg1) {
      arg0.passOp = __wbindgen_enum_GpuStencilOperation[arg1];
    },
    __wbg_set_power_preference_b42d00a8facfbade: function(arg0, arg1) {
      arg0.powerPreference = __wbindgen_enum_GpuPowerPreference[arg1];
    },
    __wbg_set_primitive_e796cf76f0ff89f3: function(arg0, arg1) {
      arg0.primitive = arg1;
    },
    __wbg_set_query_set_f030702f1b69199f: function(arg0, arg1) {
      arg0.querySet = arg1;
    },
    __wbg_set_r_6ece4d74af63364f: function(arg0, arg1) {
      arg0.r = arg1;
    },
    __wbg_set_required_features_bbab71414c45e621: function(arg0, arg1, arg2) {
      arg0.requiredFeatures = getArrayJsValueViewFromWasm0(arg1, arg2);
    },
    __wbg_set_required_limits_837f62d865e7cfac: function(arg0, arg1) {
      arg0.requiredLimits = arg1;
    },
    __wbg_set_resolve_target_gpu_texture_view_e4c1e3bbb8c27d87: function(arg0, arg1) {
      arg0.resolveTarget = arg1;
    },
    __wbg_set_resource_8fd8658b30d86ecf: function(arg0, arg1) {
      arg0.resource = arg1;
    },
    __wbg_set_resource_gpu_buffer_binding_33099b25da65b610: function(arg0, arg1) {
      arg0.resource = arg1;
    },
    __wbg_set_resource_gpu_texture_view_4cffe7bc7c8e5cbe: function(arg0, arg1) {
      arg0.resource = arg1;
    },
    __wbg_set_rows_per_image_c6d50d227e634379: function(arg0, arg1) {
      arg0.rowsPerImage = arg1 >>> 0;
    },
    __wbg_set_sample_count_481c255a12054e1d: function(arg0, arg1) {
      arg0.sampleCount = arg1 >>> 0;
    },
    __wbg_set_sample_type_ebc5fcd029513bda: function(arg0, arg1) {
      arg0.sampleType = __wbindgen_enum_GpuTextureSampleType[arg1];
    },
    __wbg_set_sampler_89cb4a7efcfc6005: function(arg0, arg1) {
      arg0.sampler = arg1;
    },
    __wbg_set_shader_location_3fb9f6a012eba494: function(arg0, arg1) {
      arg0.shaderLocation = arg1 >>> 0;
    },
    __wbg_set_size_f64_2f591b0654540477: function(arg0, arg1) {
      arg0.size = arg1;
    },
    __wbg_set_size_f64_e844c985b8f95261: function(arg0, arg1) {
      arg0.size = arg1;
    },
    __wbg_set_size_gpu_extent_3d_dict_adf57388ab1d4f18: function(arg0, arg1) {
      arg0.size = arg1;
    },
    __wbg_set_src_factor_6f2c9ec8e4d3d979: function(arg0, arg1) {
      arg0.srcFactor = __wbindgen_enum_GpuBlendFactor[arg1];
    },
    __wbg_set_stencil_back_c54d0443b8b6a957: function(arg0, arg1) {
      arg0.stencilBack = arg1;
    },
    __wbg_set_stencil_clear_value_a321b0e045bfd8c2: function(arg0, arg1) {
      arg0.stencilClearValue = arg1 >>> 0;
    },
    __wbg_set_stencil_front_3ff3f8385852efff: function(arg0, arg1) {
      arg0.stencilFront = arg1;
    },
    __wbg_set_stencil_load_op_37d20deccb26a0f1: function(arg0, arg1) {
      arg0.stencilLoadOp = __wbindgen_enum_GpuLoadOp[arg1];
    },
    __wbg_set_stencil_read_mask_021ef4271b24352c: function(arg0, arg1) {
      arg0.stencilReadMask = arg1 >>> 0;
    },
    __wbg_set_stencil_read_only_75fe66a2356d6e92: function(arg0, arg1) {
      arg0.stencilReadOnly = arg1 !== 0;
    },
    __wbg_set_stencil_store_op_501f91638dd386e6: function(arg0, arg1) {
      arg0.stencilStoreOp = __wbindgen_enum_GpuStoreOp[arg1];
    },
    __wbg_set_stencil_write_mask_ec1c12237e094bdd: function(arg0, arg1) {
      arg0.stencilWriteMask = arg1 >>> 0;
    },
    __wbg_set_step_mode_3cbbdeba1e5dfd62: function(arg0, arg1) {
      arg0.stepMode = __wbindgen_enum_GpuVertexStepMode[arg1];
    },
    __wbg_set_storage_texture_786aea7c5773b6c1: function(arg0, arg1) {
      arg0.storageTexture = arg1;
    },
    __wbg_set_store_op_678f33376d741711: function(arg0, arg1) {
      arg0.storeOp = __wbindgen_enum_GpuStoreOp[arg1];
    },
    __wbg_set_strip_index_format_70313df755145d5e: function(arg0, arg1) {
      arg0.stripIndexFormat = __wbindgen_enum_GpuIndexFormat[arg1];
    },
    __wbg_set_targets_674b33931e512fb1: function(arg0, arg1, arg2) {
      arg0.targets = getArrayJsValueViewFromWasm0(arg1, arg2);
    },
    __wbg_set_texture_95f2bfdf7767e76f: function(arg0, arg1) {
      arg0.texture = arg1;
    },
    __wbg_set_texture_a33be3fe02ac6264: function(arg0, arg1) {
      arg0.texture = arg1;
    },
    __wbg_set_timestamp_writes_de6a09f299b71b76: function(arg0, arg1) {
      arg0.timestampWrites = arg1;
    },
    __wbg_set_tone_mapping_320c1aad31db2e7f: function(arg0, arg1) {
      arg0.toneMapping = arg1;
    },
    __wbg_set_topology_b92cfe523bd9653b: function(arg0, arg1) {
      arg0.topology = __wbindgen_enum_GpuPrimitiveTopology[arg1];
    },
    __wbg_set_type_43e0092f16775979: function(arg0, arg1) {
      arg0.type = __wbindgen_enum_GpuSamplerBindingType[arg1];
    },
    __wbg_set_type_79cec55caf4cdb6d: function(arg0, arg1) {
      arg0.type = __wbindgen_enum_GpuBufferBindingType[arg1];
    },
    __wbg_set_unclipped_depth_32b7caf29fa5633d: function(arg0, arg1) {
      arg0.unclippedDepth = arg1 !== 0;
    },
    __wbg_set_usage_1ee33d98267e787d: function(arg0, arg1) {
      arg0.usage = arg1 >>> 0;
    },
    __wbg_set_usage_2365e2704b1fdb10: function(arg0, arg1) {
      arg0.usage = arg1 >>> 0;
    },
    __wbg_set_usage_d53ee6f0c7aedbfa: function(arg0, arg1) {
      arg0.usage = arg1 >>> 0;
    },
    __wbg_set_usage_f3e34822998d2147: function(arg0, arg1) {
      arg0.usage = arg1 >>> 0;
    },
    __wbg_set_vertex_77ed7a1229239b5a: function(arg0, arg1) {
      arg0.vertex = arg1;
    },
    __wbg_set_view_dimension_893e2d16561e56e8: function(arg0, arg1) {
      arg0.viewDimension = __wbindgen_enum_GpuTextureViewDimension[arg1];
    },
    __wbg_set_view_dimension_f2c5fe4bf927c3fe: function(arg0, arg1) {
      arg0.viewDimension = __wbindgen_enum_GpuTextureViewDimension[arg1];
    },
    __wbg_set_view_formats_427069064d8b7139: function(arg0, arg1, arg2) {
      arg0.viewFormats = getArrayJsValueViewFromWasm0(arg1, arg2);
    },
    __wbg_set_view_formats_9c2f01a6f3b365c7: function(arg0, arg1, arg2) {
      arg0.viewFormats = getArrayJsValueViewFromWasm0(arg1, arg2);
    },
    __wbg_set_view_gpu_texture_view_35f4655788535c4d: function(arg0, arg1) {
      arg0.view = arg1;
    },
    __wbg_set_view_gpu_texture_view_a532c825c52042c0: function(arg0, arg1) {
      arg0.view = arg1;
    },
    __wbg_set_visibility_d8a6821789538c25: function(arg0, arg1) {
      arg0.visibility = arg1 >>> 0;
    },
    __wbg_set_width_0ca908d167690992: function(arg0, arg1) {
      arg0.width = arg1 >>> 0;
    },
    __wbg_set_width_b0e1267db4b196b5: function(arg0, arg1) {
      arg0.width = arg1 >>> 0;
    },
    __wbg_set_width_b20525f5f4df4eb8: function(arg0, arg1) {
      arg0.width = arg1 >>> 0;
    },
    __wbg_set_write_mask_42d89f182ade6b2d: function(arg0, arg1) {
      arg0.writeMask = arg1 >>> 0;
    },
    __wbg_set_x_f470b03dd54724cd: function(arg0, arg1) {
      arg0.x = arg1 >>> 0;
    },
    __wbg_set_y_4c44eb40ebca5bfc: function(arg0, arg1) {
      arg0.y = arg1 >>> 0;
    },
    __wbg_set_z_2e6820ef0f5821ed: function(arg0, arg1) {
      arg0.z = arg1 >>> 0;
    },
    __wbg_static_accessor_GLOBAL_266715b9d96ba635: function() {
      const ret = typeof global === "undefined" ? null : global;
      return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    },
    __wbg_static_accessor_GLOBAL_THIS_10fb7dc1ae063179: function() {
      const ret = typeof globalThis === "undefined" ? null : globalThis;
      return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    },
    __wbg_static_accessor_SELF_0b583911f537483a: function() {
      const ret = typeof self === "undefined" ? null : self;
      return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    },
    __wbg_static_accessor_WINDOW_d7f903d1508cbdc4: function() {
      const ret = typeof window === "undefined" ? null : window;
      return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    },
    __wbg_subgroupMaxSize_b43be0aa16182403: function(arg0) {
      const ret = arg0.subgroupMaxSize;
      return ret;
    },
    __wbg_subgroupMinSize_03feb6ee0cda6775: function(arg0) {
      const ret = arg0.subgroupMinSize;
      return ret;
    },
    __wbg_submit_077c85cc28e36892: function(arg0, arg1, arg2) {
      arg0.submit(getArrayJsValueViewFromWasm0(arg1, arg2));
    },
    __wbg_then_c949d5a25a4e78f8: function(arg0, arg1, arg2) {
      const ret = arg0.then(arg1, arg2);
      return ret;
    },
    __wbg_then_e71170d78fcf8954: function(arg0, arg1) {
      const ret = arg0.then(arg1);
      return ret;
    },
    __wbg_unconfigure_835307f58dc68d80: function(arg0) {
      arg0.unconfigure();
    },
    __wbg_width_3d0dce3d9892e35e: function(arg0) {
      const ret = arg0.width;
      return ret;
    },
    __wbg_width_7a1b335e4a553dc3: function(arg0) {
      const ret = arg0.width;
      return ret;
    },
    __wbg_writeBuffer_f4bb3f54adfe1330: function() {
      return handleError(function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
        arg0.writeBuffer(arg1, arg2, getArrayU8FromWasm0(arg3, arg4), arg5, arg6);
      }, arguments);
    },
    __wbg_writeTexture_30e592e8c061c3d9: function() {
      return handleError(function(arg0, arg1, arg2, arg3, arg4, arg5) {
        arg0.writeTexture(arg1, getArrayU8FromWasm0(arg2, arg3), arg4, arg5);
      }, arguments);
    },
    __wbindgen_generic_0000000000000001: function(arg0, arg1) {
      const ret = makeMutClosure(arg0, arg1, wasm_bindgen_16ce60f5be4e30c6___convert__closures_____invoke___wasm_bindgen_16ce60f5be4e30c6___JsValue__core_608f92abc48d28da___result__Result_____wasm_bindgen_16ce60f5be4e30c6___JsError___true_);
      return ret;
    },
    __wbindgen_generic_0000000000000002: function(arg0, arg1) {
      const ret = makeMutClosure(arg0, arg1, wasm_bindgen_16ce60f5be4e30c6___convert__closures_____invoke___wasm_bindgen_16ce60f5be4e30c6___sys__JsNullable_wgpu_8e86242f95ca2672___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_608f92abc48d28da___result__Result_____wasm_bindgen_16ce60f5be4e30c6___JsError___true_);
      return ret;
    },
    __wbindgen_generic_0000000000000003: function(arg0, arg1) {
      const ret = makeMutClosure(arg0, arg1, wasm_bindgen_16ce60f5be4e30c6___convert__closures_____invoke___wasm_bindgen_16ce60f5be4e30c6___sys__JsNullable_wgpu_8e86242f95ca2672___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_608f92abc48d28da___result__Result_____wasm_bindgen_16ce60f5be4e30c6___JsError___true__28);
      return ret;
    },
    __wbindgen_generic_0000000000000004: function(arg0, arg1) {
      const ret = makeMutClosure(arg0, arg1, wasm_bindgen_16ce60f5be4e30c6___convert__closures_____invoke___wasm_bindgen_16ce60f5be4e30c6___sys__JsNullable_wgpu_8e86242f95ca2672___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_608f92abc48d28da___result__Result_____wasm_bindgen_16ce60f5be4e30c6___JsError___true__29);
      return ret;
    },
    __wbindgen_generic_0000000000000005: function(arg0) {
      const ret = arg0;
      return ret;
    },
    __wbindgen_generic_0000000000000006: function(arg0, arg1) {
      const ret = getStringFromWasm0(arg0, arg1);
      return ret;
    },
    __wbindgen_init_externref_table: function() {
      const table = wasm.__wbindgen_externrefs;
      const offset = table.grow(4);
      table.set(0, void 0);
      table.set(offset + 0, void 0);
      table.set(offset + 1, null);
      table.set(offset + 2, true);
      table.set(offset + 3, false);
    }
  };
  return {
    __proto__: null,
    "./mb_host_bg.js": import0
  };
}
function wasm_bindgen_16ce60f5be4e30c6___convert__closures_____invoke___wasm_bindgen_16ce60f5be4e30c6___JsValue__core_608f92abc48d28da___result__Result_____wasm_bindgen_16ce60f5be4e30c6___JsError___true_(arg0, arg1, arg2) {
  const ret = wasm.wasm_bindgen_16ce60f5be4e30c6___convert__closures_____invoke___wasm_bindgen_16ce60f5be4e30c6___JsValue__core_608f92abc48d28da___result__Result_____wasm_bindgen_16ce60f5be4e30c6___JsError___true_(arg0, arg1, arg2);
  if (ret[1]) {
    throw takeFromExternrefTable0(ret[0]);
  }
}
function wasm_bindgen_16ce60f5be4e30c6___convert__closures_____invoke___wasm_bindgen_16ce60f5be4e30c6___sys__JsNullable_wgpu_8e86242f95ca2672___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_608f92abc48d28da___result__Result_____wasm_bindgen_16ce60f5be4e30c6___JsError___true_(arg0, arg1, arg2) {
  const ret = wasm.wasm_bindgen_16ce60f5be4e30c6___convert__closures_____invoke___wasm_bindgen_16ce60f5be4e30c6___sys__JsNullable_wgpu_8e86242f95ca2672___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_608f92abc48d28da___result__Result_____wasm_bindgen_16ce60f5be4e30c6___JsError___true_(arg0, arg1, arg2);
  if (ret[1]) {
    throw takeFromExternrefTable0(ret[0]);
  }
}
function wasm_bindgen_16ce60f5be4e30c6___convert__closures_____invoke___wasm_bindgen_16ce60f5be4e30c6___sys__JsNullable_wgpu_8e86242f95ca2672___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_608f92abc48d28da___result__Result_____wasm_bindgen_16ce60f5be4e30c6___JsError___true__28(arg0, arg1, arg2) {
  const ret = wasm.wasm_bindgen_16ce60f5be4e30c6___convert__closures_____invoke___wasm_bindgen_16ce60f5be4e30c6___sys__JsNullable_wgpu_8e86242f95ca2672___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_608f92abc48d28da___result__Result_____wasm_bindgen_16ce60f5be4e30c6___JsError___true__28(arg0, arg1, arg2);
  if (ret[1]) {
    throw takeFromExternrefTable0(ret[0]);
  }
}
function wasm_bindgen_16ce60f5be4e30c6___convert__closures_____invoke___wasm_bindgen_16ce60f5be4e30c6___sys__JsNullable_wgpu_8e86242f95ca2672___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_608f92abc48d28da___result__Result_____wasm_bindgen_16ce60f5be4e30c6___JsError___true__29(arg0, arg1, arg2) {
  const ret = wasm.wasm_bindgen_16ce60f5be4e30c6___convert__closures_____invoke___wasm_bindgen_16ce60f5be4e30c6___sys__JsNullable_wgpu_8e86242f95ca2672___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_608f92abc48d28da___result__Result_____wasm_bindgen_16ce60f5be4e30c6___JsError___true__29(arg0, arg1, arg2);
  if (ret[1]) {
    throw takeFromExternrefTable0(ret[0]);
  }
}
function wasm_bindgen_16ce60f5be4e30c6___convert__closures_____invoke___js_sys_ad02f14a055e7bf2___Function_fn_wasm_bindgen_16ce60f5be4e30c6___JsValue_____wasm_bindgen_16ce60f5be4e30c6___sys__Undefined___js_sys_ad02f14a055e7bf2___Function_fn_wasm_bindgen_16ce60f5be4e30c6___JsValue_____wasm_bindgen_16ce60f5be4e30c6___sys__Undefined_______true_(arg0, arg1, arg2, arg3) {
  wasm.wasm_bindgen_16ce60f5be4e30c6___convert__closures_____invoke___js_sys_ad02f14a055e7bf2___Function_fn_wasm_bindgen_16ce60f5be4e30c6___JsValue_____wasm_bindgen_16ce60f5be4e30c6___sys__Undefined___js_sys_ad02f14a055e7bf2___Function_fn_wasm_bindgen_16ce60f5be4e30c6___JsValue_____wasm_bindgen_16ce60f5be4e30c6___sys__Undefined_______true_(arg0, arg1, arg2, arg3);
}
var __wbindgen_enum_GpuAddressMode = ["clamp-to-edge", "repeat", "mirror-repeat"];
var __wbindgen_enum_GpuAutoLayoutMode = ["auto"];
var __wbindgen_enum_GpuBlendFactor = ["zero", "one", "src", "one-minus-src", "src-alpha", "one-minus-src-alpha", "dst", "one-minus-dst", "dst-alpha", "one-minus-dst-alpha", "src-alpha-saturated", "constant", "one-minus-constant", "src1", "one-minus-src1", "src1-alpha", "one-minus-src1-alpha"];
var __wbindgen_enum_GpuBlendOperation = ["add", "subtract", "reverse-subtract", "min", "max"];
var __wbindgen_enum_GpuBufferBindingType = ["uniform", "storage", "read-only-storage"];
var __wbindgen_enum_GpuCanvasAlphaMode = ["opaque", "premultiplied"];
var __wbindgen_enum_GpuCanvasToneMappingMode = ["standard", "extended"];
var __wbindgen_enum_GpuCompareFunction = ["never", "less", "equal", "less-equal", "greater", "not-equal", "greater-equal", "always"];
var __wbindgen_enum_GpuCullMode = ["none", "front", "back"];
var __wbindgen_enum_GpuFilterMode = ["nearest", "linear"];
var __wbindgen_enum_GpuFrontFace = ["ccw", "cw"];
var __wbindgen_enum_GpuIndexFormat = ["uint16", "uint32"];
var __wbindgen_enum_GpuLoadOp = ["load", "clear"];
var __wbindgen_enum_GpuMipmapFilterMode = ["nearest", "linear"];
var __wbindgen_enum_GpuPowerPreference = ["low-power", "high-performance"];
var __wbindgen_enum_GpuPrimitiveTopology = ["point-list", "line-list", "line-strip", "triangle-list", "triangle-strip"];
var __wbindgen_enum_GpuSamplerBindingType = ["filtering", "non-filtering", "comparison"];
var __wbindgen_enum_GpuStencilOperation = ["keep", "zero", "replace", "invert", "increment-clamp", "decrement-clamp", "increment-wrap", "decrement-wrap"];
var __wbindgen_enum_GpuStorageTextureAccess = ["write-only", "read-only", "read-write"];
var __wbindgen_enum_GpuStoreOp = ["store", "discard"];
var __wbindgen_enum_GpuTextureAspect = ["all", "stencil-only", "depth-only"];
var __wbindgen_enum_GpuTextureDimension = ["1d", "2d", "3d"];
var __wbindgen_enum_GpuTextureFormat = ["r8unorm", "r8snorm", "r8uint", "r8sint", "r16unorm", "r16snorm", "r16uint", "r16sint", "r16float", "rg8unorm", "rg8snorm", "rg8uint", "rg8sint", "r32uint", "r32sint", "r32float", "rg16unorm", "rg16snorm", "rg16uint", "rg16sint", "rg16float", "rgba8unorm", "rgba8unorm-srgb", "rgba8snorm", "rgba8uint", "rgba8sint", "bgra8unorm", "bgra8unorm-srgb", "rgb9e5ufloat", "rgb10a2uint", "rgb10a2unorm", "rg11b10ufloat", "rg32uint", "rg32sint", "rg32float", "rgba16unorm", "rgba16snorm", "rgba16uint", "rgba16sint", "rgba16float", "rgba32uint", "rgba32sint", "rgba32float", "stencil8", "depth16unorm", "depth24plus", "depth24plus-stencil8", "depth32float", "depth32float-stencil8", "bc1-rgba-unorm", "bc1-rgba-unorm-srgb", "bc2-rgba-unorm", "bc2-rgba-unorm-srgb", "bc3-rgba-unorm", "bc3-rgba-unorm-srgb", "bc4-r-unorm", "bc4-r-snorm", "bc5-rg-unorm", "bc5-rg-snorm", "bc6h-rgb-ufloat", "bc6h-rgb-float", "bc7-rgba-unorm", "bc7-rgba-unorm-srgb", "etc2-rgb8unorm", "etc2-rgb8unorm-srgb", "etc2-rgb8a1unorm", "etc2-rgb8a1unorm-srgb", "etc2-rgba8unorm", "etc2-rgba8unorm-srgb", "eac-r11unorm", "eac-r11snorm", "eac-rg11unorm", "eac-rg11snorm", "astc-4x4-unorm", "astc-4x4-unorm-srgb", "astc-5x4-unorm", "astc-5x4-unorm-srgb", "astc-5x5-unorm", "astc-5x5-unorm-srgb", "astc-6x5-unorm", "astc-6x5-unorm-srgb", "astc-6x6-unorm", "astc-6x6-unorm-srgb", "astc-8x5-unorm", "astc-8x5-unorm-srgb", "astc-8x6-unorm", "astc-8x6-unorm-srgb", "astc-8x8-unorm", "astc-8x8-unorm-srgb", "astc-10x5-unorm", "astc-10x5-unorm-srgb", "astc-10x6-unorm", "astc-10x6-unorm-srgb", "astc-10x8-unorm", "astc-10x8-unorm-srgb", "astc-10x10-unorm", "astc-10x10-unorm-srgb", "astc-12x10-unorm", "astc-12x10-unorm-srgb", "astc-12x12-unorm", "astc-12x12-unorm-srgb"];
var __wbindgen_enum_GpuTextureSampleType = ["float", "unfilterable-float", "depth", "sint", "uint"];
var __wbindgen_enum_GpuTextureViewDimension = ["1d", "2d", "2d-array", "cube", "cube-array", "3d"];
var __wbindgen_enum_GpuVertexFormat = ["uint8", "uint8x2", "uint8x4", "sint8", "sint8x2", "sint8x4", "unorm8", "unorm8x2", "unorm8x4", "snorm8", "snorm8x2", "snorm8x4", "uint16", "uint16x2", "uint16x4", "sint16", "sint16x2", "sint16x4", "unorm16", "unorm16x2", "unorm16x4", "snorm16", "snorm16x2", "snorm16x4", "float16", "float16x2", "float16x4", "float32", "float32x2", "float32x3", "float32x4", "uint32", "uint32x2", "uint32x3", "uint32x4", "sint32", "sint32x2", "sint32x3", "sint32x4", "unorm10-10-10-2", "unorm8x4-bgra"];
var __wbindgen_enum_GpuVertexStepMode = ["vertex", "instance"];
function addToExternrefTable0(obj) {
  const idx = wasm.__externref_table_alloc();
  wasm.__wbindgen_externrefs.set(idx, obj);
  return idx;
}
var CLOSURE_DTORS = typeof FinalizationRegistry === "undefined" ? { register: () => {
}, unregister: () => {
} } : new FinalizationRegistry((state) => wasm.__wbindgen_destroy_closure(state.a, state.b));
function debugString(val) {
  const type = typeof val;
  if (type == "number" || type == "boolean" || val == null) {
    return `${val}`;
  }
  if (type == "string") {
    return `"${val}"`;
  }
  if (type == "symbol") {
    const description = val.description;
    if (description == null) {
      return "Symbol";
    } else {
      return `Symbol(${description})`;
    }
  }
  if (type == "function") {
    const name = val.name;
    if (typeof name == "string" && name.length > 0) {
      return `Function(${name})`;
    } else {
      return "Function";
    }
  }
  if (Array.isArray(val)) {
    const length = val.length;
    let debug = "[";
    if (length > 0) {
      debug += debugString(val[0]);
    }
    for (let i = 1; i < length; i++) {
      debug += ", " + debugString(val[i]);
    }
    debug += "]";
    return debug;
  }
  const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
  let className;
  if (builtInMatches && builtInMatches.length > 1) {
    className = builtInMatches[1];
  } else {
    return toString.call(val);
  }
  if (className == "Object") {
    try {
      return "Object(" + JSON.stringify(val) + ")";
    } catch (_) {
      return "Object";
    }
  }
  if (val instanceof Error) {
    return `${val.name}: ${val.message}
${val.stack}`;
  }
  return className;
}
function getArrayJsValueViewFromWasm0(ptr, len) {
  ptr = ptr >>> 0;
  const mem = getDataViewMemory0();
  const result = [];
  for (let i = ptr; i < ptr + 4 * len; i += 4) {
    result.push(wasm.__wbindgen_externrefs.get(mem.getUint32(i, true)));
  }
  return result;
}
function getArrayU32FromWasm0(ptr, len) {
  ptr = ptr >>> 0;
  return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}
function getArrayU8FromWasm0(ptr, len) {
  ptr = ptr >>> 0;
  return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}
var cachedDataViewMemory0 = null;
function getDataViewMemory0() {
  if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || cachedDataViewMemory0.buffer.detached === void 0 && cachedDataViewMemory0.buffer !== wasm.memory.buffer) {
    cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
  }
  return cachedDataViewMemory0;
}
function getStringFromWasm0(ptr, len) {
  return decodeText(ptr >>> 0, len);
}
var cachedUint32ArrayMemory0 = null;
function getUint32ArrayMemory0() {
  if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
    cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
  }
  return cachedUint32ArrayMemory0;
}
var cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
  if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
    cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
  }
  return cachedUint8ArrayMemory0;
}
function handleError(f, args) {
  try {
    return f.apply(this, args);
  } catch (e) {
    const idx = addToExternrefTable0(e);
    wasm.__wbindgen_exn_store(idx);
  }
}
function isLikeNone(x) {
  return x === void 0 || x === null;
}
function makeMutClosure(arg0, arg1, f) {
  const state = { a: arg0, b: arg1, cnt: 1 };
  const real = (...args) => {
    state.cnt++;
    const a = state.a;
    state.a = 0;
    try {
      return f(a, state.b, ...args);
    } finally {
      state.a = a;
      real._wbg_cb_unref();
    }
  };
  real._wbg_cb_unref = () => {
    if (--state.cnt === 0) {
      wasm.__wbindgen_destroy_closure(state.a, state.b);
      state.a = 0;
      CLOSURE_DTORS.unregister(state);
    }
  };
  CLOSURE_DTORS.register(real, state, state);
  return real;
}
function passStringToWasm0(arg, malloc, realloc) {
  if (realloc === void 0) {
    const buf = cachedTextEncoder.encode(arg);
    const ptr2 = malloc(buf.length, 1) >>> 0;
    getUint8ArrayMemory0().subarray(ptr2, ptr2 + buf.length).set(buf);
    WASM_VECTOR_LEN = buf.length;
    return ptr2;
  }
  let len = arg.length;
  let ptr = malloc(len, 1) >>> 0;
  const mem = getUint8ArrayMemory0();
  let offset = 0;
  for (; offset < len; offset++) {
    const code = arg.charCodeAt(offset);
    if (code > 127) break;
    mem[ptr + offset] = code;
  }
  if (offset !== len) {
    if (offset !== 0) {
      arg = arg.slice(offset);
    }
    ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
    const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
    const ret = cachedTextEncoder.encodeInto(arg, view);
    offset += ret.written;
    ptr = realloc(ptr, len, offset, 1) >>> 0;
  }
  WASM_VECTOR_LEN = offset;
  return ptr;
}
function takeFromExternrefTable0(idx) {
  const value = wasm.__wbindgen_externrefs.get(idx);
  wasm.__externref_table_dealloc(idx);
  return value;
}
var cachedTextDecoder = new TextDecoder("utf-8", { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
var MAX_SAFARI_DECODE_BYTES = 2146435072;
var numBytesDecoded = 0;
function decodeText(ptr, len) {
  numBytesDecoded += len;
  if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
    cachedTextDecoder = new TextDecoder("utf-8", { ignoreBOM: true, fatal: true });
    cachedTextDecoder.decode();
    numBytesDecoded = len;
  }
  return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}
var cachedTextEncoder = new TextEncoder();
if (!("encodeInto" in cachedTextEncoder)) {
  cachedTextEncoder.encodeInto = function(arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
      read: arg.length,
      written: buf.length
    };
  };
}
var WASM_VECTOR_LEN = 0;
var wasmModule;
var wasmInstance;
var wasm;
function __wbg_finalize_init(instance, module) {
  wasmInstance = instance;
  wasm = instance.exports;
  wasmModule = module;
  cachedDataViewMemory0 = null;
  cachedUint32ArrayMemory0 = null;
  cachedUint8ArrayMemory0 = null;
  wasm.__wbindgen_start();
  return wasm;
}
async function __wbg_load(module, imports) {
  if (typeof Response === "function" && module instanceof Response) {
    if (!module.ok) {
      throw new Error(`failed to fetch Wasm: ${module.status} ${module.statusText} fetching '${module.url}'`);
    }
    if (typeof WebAssembly.instantiateStreaming === "function") {
      try {
        return await WebAssembly.instantiateStreaming(module, imports);
      } catch (e) {
        const validResponse = expectedResponseType(module.type);
        if (validResponse && module.headers.get("Content-Type") !== "application/wasm") {
          console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);
        } else {
          throw e;
        }
      }
    }
    const bytes = await module.arrayBuffer();
    return await WebAssembly.instantiate(bytes, imports);
  } else {
    const instance = await WebAssembly.instantiate(module, imports);
    if (instance instanceof WebAssembly.Instance) {
      return { instance, module };
    } else {
      return instance;
    }
  }
  function expectedResponseType(type) {
    switch (type) {
      case "basic":
      case "cors":
      case "default":
        return true;
    }
    return false;
  }
}
async function __wbg_init(module_or_path) {
  if (wasm !== void 0) return wasm;
  if (module_or_path !== void 0) {
    if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
      ({ module_or_path } = module_or_path);
    } else {
      console.warn("using deprecated parameters for the initialization function; pass a single object instead");
    }
  }
  if (module_or_path === void 0) {
    module_or_path = new URL("mb_host_bg.wasm", import.meta.url);
  }
  const imports = __wbg_get_imports();
  if (typeof module_or_path === "string" || typeof Request === "function" && module_or_path instanceof Request || typeof URL === "function" && module_or_path instanceof URL) {
    module_or_path = fetch(module_or_path);
  }
  const { instance, module } = await __wbg_load(await module_or_path, imports);
  return __wbg_finalize_init(instance, module);
}

// src/abi.json
var abi_default = {
  abi: 0,
  imports: [
    {
      name: "mb_log",
      params: [
        "i32",
        "i32",
        "i32"
      ],
      requires: null,
      results: []
    },
    {
      name: "mb_time",
      params: [],
      requires: null,
      results: [
        "f64"
      ]
    },
    {
      name: "mb_rand_seed",
      params: [],
      requires: null,
      results: [
        "i64"
      ]
    },
    {
      name: "mb_daily_seed",
      params: [],
      requires: null,
      results: [
        "i64"
      ]
    },
    {
      name: "mb_round",
      params: [
        "i32"
      ],
      requires: null,
      results: []
    },
    {
      name: "mb_player_id",
      params: [
        "i32"
      ],
      requires: null,
      results: [
        "i32"
      ]
    },
    {
      name: "mb_locale",
      params: [
        "i32",
        "i32"
      ],
      requires: null,
      results: [
        "i32"
      ]
    },
    {
      name: "mb_screen",
      params: [
        "i32"
      ],
      requires: null,
      results: []
    },
    {
      name: "mb_asset_load",
      params: [
        "i32",
        "i32"
      ],
      requires: null,
      results: [
        "i32"
      ]
    },
    {
      name: "mb_asset_state",
      params: [
        "i32"
      ],
      requires: null,
      results: [
        "i32"
      ]
    },
    {
      name: "mb_asset_read",
      params: [
        "i32",
        "i32",
        "i32"
      ],
      requires: null,
      results: [
        "i32"
      ]
    },
    {
      name: "mb_input_poll",
      params: [
        "i32",
        "i32"
      ],
      requires: null,
      results: [
        "i32"
      ]
    },
    {
      name: "mb2d_image",
      params: [
        "i32"
      ],
      requires: {
        stdlib: "mb2d"
      },
      results: [
        "i32"
      ]
    },
    {
      name: "mb2d_font",
      params: [
        "i32"
      ],
      requires: {
        stdlib: "mb2d"
      },
      results: [
        "i32"
      ]
    },
    {
      name: "mb2d_clear",
      params: [
        "i32"
      ],
      requires: {
        stdlib: "mb2d"
      },
      results: []
    },
    {
      name: "mb2d_push",
      params: [],
      requires: {
        stdlib: "mb2d"
      },
      results: []
    },
    {
      name: "mb2d_pop",
      params: [],
      requires: {
        stdlib: "mb2d"
      },
      results: []
    },
    {
      name: "mb2d_translate",
      params: [
        "f32",
        "f32"
      ],
      requires: {
        stdlib: "mb2d"
      },
      results: []
    },
    {
      name: "mb2d_rotate",
      params: [
        "f32"
      ],
      requires: {
        stdlib: "mb2d"
      },
      results: []
    },
    {
      name: "mb2d_scale",
      params: [
        "f32",
        "f32"
      ],
      requires: {
        stdlib: "mb2d"
      },
      results: []
    },
    {
      name: "mb2d_blend",
      params: [
        "i32"
      ],
      requires: {
        stdlib: "mb2d"
      },
      results: []
    },
    {
      name: "mb2d_rect",
      params: [
        "f32",
        "f32",
        "f32",
        "f32",
        "i32"
      ],
      requires: {
        stdlib: "mb2d"
      },
      results: []
    },
    {
      name: "mb2d_rect_gradient",
      params: [
        "f32",
        "f32",
        "f32",
        "f32",
        "i32",
        "i32"
      ],
      requires: {
        stdlib: "mb2d"
      },
      results: []
    },
    {
      name: "mb2d_circle",
      params: [
        "f32",
        "f32",
        "f32",
        "i32"
      ],
      requires: {
        stdlib: "mb2d"
      },
      results: []
    },
    {
      name: "mb2d_line",
      params: [
        "f32",
        "f32",
        "f32",
        "f32",
        "f32",
        "i32"
      ],
      requires: {
        stdlib: "mb2d"
      },
      results: []
    },
    {
      name: "mb2d_poly",
      params: [
        "i32",
        "i32",
        "i32"
      ],
      requires: {
        stdlib: "mb2d"
      },
      results: []
    },
    {
      name: "mb2d_sprite",
      params: [
        "i32",
        "f32",
        "f32",
        "f32",
        "f32",
        "f32",
        "f32",
        "f32",
        "f32",
        "i32"
      ],
      requires: {
        stdlib: "mb2d"
      },
      results: []
    },
    {
      name: "mb2d_text",
      params: [
        "i32",
        "f32",
        "f32",
        "f32",
        "i32",
        "i32",
        "i32"
      ],
      requires: {
        stdlib: "mb2d"
      },
      results: []
    },
    {
      name: "mb2d_measure",
      params: [
        "i32",
        "f32",
        "i32",
        "i32"
      ],
      requires: {
        stdlib: "mb2d"
      },
      results: [
        "f32"
      ]
    },
    {
      name: "mb_sound",
      params: [
        "i32"
      ],
      requires: null,
      results: [
        "i32"
      ]
    },
    {
      name: "mb_play",
      params: [
        "i32",
        "f32",
        "f32",
        "f32",
        "i32"
      ],
      requires: null,
      results: [
        "i32"
      ]
    },
    {
      name: "mb_voice_set",
      params: [
        "i32",
        "f32",
        "f32",
        "f32"
      ],
      requires: null,
      results: []
    },
    {
      name: "mb_voice_stop",
      params: [
        "i32"
      ],
      requires: null,
      results: []
    },
    {
      name: "mb_tilt",
      params: [
        "i32"
      ],
      requires: {
        sensor: "tilt"
      },
      results: [
        "i32"
      ]
    },
    {
      name: "mb_motion",
      params: [
        "i32"
      ],
      requires: {
        sensor: "motion"
      },
      results: [
        "i32"
      ]
    },
    {
      name: "mb_loudness",
      params: [],
      requires: {
        sensor: "loudness"
      },
      results: [
        "f32"
      ]
    },
    {
      name: "mb_light",
      params: [],
      requires: {
        sensor: "light"
      },
      results: [
        "f32"
      ]
    },
    {
      name: "mb_haptic",
      params: [
        "i32"
      ],
      requires: {
        sensor: "haptics"
      },
      results: []
    },
    {
      name: "mb_store_get",
      params: [
        "i32",
        "i32",
        "i32",
        "i32"
      ],
      requires: {
        capability: "store"
      },
      results: [
        "i32"
      ]
    },
    {
      name: "mb_store_set",
      params: [
        "i32",
        "i32",
        "i32",
        "i32"
      ],
      requires: {
        capability: "store"
      },
      results: [
        "i32"
      ]
    },
    {
      name: "mb_score_submit",
      params: [
        "i32",
        "i64"
      ],
      requires: {
        capability: "score"
      },
      results: [
        "i32"
      ]
    },
    {
      name: "mb_score_show",
      params: [
        "i32"
      ],
      requires: {
        capability: "score"
      },
      results: []
    },
    {
      name: "mb_text_begin",
      params: [
        "i32",
        "i32",
        "i32"
      ],
      requires: {
        capability: "text_input"
      },
      results: []
    },
    {
      name: "mb_text_end",
      params: [],
      requires: {
        capability: "text_input"
      },
      results: []
    },
    {
      name: "mb_link_open",
      params: [
        "i32",
        "i32"
      ],
      requires: {
        capability: "links"
      },
      results: [
        "i32"
      ]
    }
  ]
};

// src/bridge.ts
var native = window.webkit?.messageHandlers?.mb;
function post(m) {
  if (native) native.postMessage(m);
  if (m.op !== "heartbeat") console.log("[mb]", m.op, JSON.stringify(m));
}
var hasNative = native !== void 0;

// src/abi.ts
var HOST_DIRECT = [
  "mb2d_clear",
  "mb2d_push",
  "mb2d_pop",
  "mb2d_translate",
  "mb2d_rotate",
  "mb2d_scale",
  "mb2d_blend",
  "mb2d_rect",
  "mb2d_rect_gradient",
  "mb2d_circle",
  "mb2d_line",
  "mb2d_sprite"
];
var STORE_QUOTA = 256 * 1024;
var STORE_KEY_MAX = 64;
function declared(req, m) {
  if (!req) return true;
  if (req.stdlib) return req.stdlib in m.stdlib;
  if (req.sensor) return m.sensors.includes(req.sensor);
  if (req.capability) return m.capabilities.includes(req.capability);
  return false;
}
function base64(b) {
  let s = "";
  for (const c of b) s += String.fromCharCode(c);
  return btoa(s);
}
function buildImports(s, host, m) {
  const utf8 = new TextDecoder("utf-8", { fatal: false });
  const enc = new TextEncoder();
  const bytes = () => new Uint8Array(s.memory().buffer);
  const view = () => new DataView(s.memory().buffer);
  const slice = (ptr, len) => bytes().subarray(ptr, ptr + len);
  const str = (ptr, len) => utf8.decode(slice(ptr, len));
  const toHost = (data) => {
    const at = host.host_scratch(data.length);
    new Uint8Array(host.memory.buffer, at, data.length).set(data);
  };
  const boards = new Set((m.scores ?? []).map((b) => b.board));
  const impl = {
    // 5.1 sys
    mb_log: ((level, ptr, len) => s.log(level, str(ptr, len))),
    mb_time: (() => s.gameTime),
    mb_rand_seed: (() => BigInt.asIntN(64, s.seed)),
    mb_daily_seed: (() => BigInt.asIntN(64, s.dailySeed)),
    mb_round: ((state) => {
      if (state < 0 || state > 2 || state === s.round) return;
      s.round = state;
      if (!s.replaying) post({ op: "round", state });
    }),
    mb_player_id: ((ptr) => {
      bytes().set(s.playerId, ptr);
      return s.playerId.length;
    }),
    mb_locale: ((ptr, cap) => {
      const b = enc.encode(s.locale);
      bytes().set(b.subarray(0, cap), ptr);
      return b.length;
    }),
    mb_screen: ((ptr) => {
      const v = view();
      s.screen.forEach((x, i) => v.setFloat32(ptr + i * 4, x, true));
    }),
    mb_asset_load: ((ptr, len) => s.assets.load(str(ptr, len))),
    mb_asset_state: ((h) => s.assets.state(h)),
    mb_asset_read: ((h, ptr, cap) => {
      const st = s.assets.state(h);
      if (st === 0) return -6 /* NotReady */;
      if (st < 0) return st;
      const data = s.assets.data(h);
      if (!data) return -1 /* InvalidHandle */;
      bytes().set(data.subarray(0, Math.max(0, cap)), ptr);
      return data.length;
    }),
    // 5.2 input
    mb_input_poll: ((ptr, cap) => s.pollInput(view(), ptr, cap)),
    // 5.3 mb2d, memory-passing calls
    mb2d_poly: ((ptr, n, rgba) => {
      if (n < 3 || n > 512) return;
      toHost(slice(ptr, n * 8));
      host.mbh_poly(n, rgba);
    }),
    mb2d_text: ((font, size, x, y, rgba, ptr, len) => {
      toHost(slice(ptr, Math.min(len, 4096)));
      host.mbh_text(font, size, x, y, rgba, Math.min(len, 4096));
    }),
    mb2d_measure: ((font, size, ptr, len) => {
      toHost(slice(ptr, Math.min(len, 4096)));
      return host.mbh_measure(font, size, Math.min(len, 4096));
    }),
    mb2d_image: ((asset) => {
      const st = s.assets.state(asset);
      if (st === 0) return -6 /* NotReady */;
      const data = s.assets.data(asset);
      if (st < 0 || !data) return -1 /* InvalidHandle */;
      toHost(data);
      return host.mbh_image(data.length);
    }),
    // Games can't bundle fonts yet; use the host font ids 0–2 (SPEC §5.3).
    mb2d_font: (() => -7 /* Unsupported */),
    // 5.6 audio
    mb_sound: ((asset) => {
      const st = s.assets.state(asset);
      if (st === 0) return -6 /* NotReady */;
      const data = s.assets.data(asset);
      if (st < 0 || !data) return -1 /* InvalidHandle */;
      return s.audio.load(data);
    }),
    mb_play: ((sound, vol, pan, pitch, loop) => s.audio.play(sound, vol, pan, pitch, loop !== 0)),
    mb_voice_set: ((voice, vol, pan, pitch) => s.audio.update(voice, vol, pan, pitch)),
    mb_voice_stop: ((voice) => s.audio.stop(voice)),
    // 5.7 sensors
    mb_tilt: ((ptr) => {
      if (!s.tilt) return -6 /* NotReady */;
      const v = view();
      s.tilt.forEach((x, i) => v.setFloat32(ptr + i * 4, x, true));
      return 0;
    }),
    mb_motion: ((ptr) => {
      if (!s.motion) return -6 /* NotReady */;
      const v = view();
      s.motion.forEach((x, i) => v.setFloat32(ptr + i * 4, x, true));
      return 0;
    }),
    // −1 until a reading arrives or when the mic is unavailable (SPEC §5.7).
    mb_loudness: (() => s.loudness ?? -1),
    // Output only, so it never affects determinism; the native side decides whether to play it.
    mb_haptic: ((kind) => {
      if (kind >= 0 && kind <= 3 && !s.replaying) post({ op: "haptic", kind });
    }),
    // 5.8 store
    mb_store_get: ((kp, kl, bp, cap) => {
      const v = s.store.get(str(kp, kl));
      if (!v) return -1;
      bytes().set(v.subarray(0, Math.max(0, cap)), bp);
      return v.length;
    }),
    mb_store_set: ((kp, kl, vp, vl) => {
      if (kl === 0 || kl > STORE_KEY_MAX) return -2 /* InvalidArgument */;
      const key = str(kp, kl);
      const value = slice(vp, vl).slice();
      const old = s.store.get(key);
      const size = s.storeBytes() - (old ? key.length + old.length : 0) + (vl ? key.length + vl : 0);
      if (size > STORE_QUOTA) return -4 /* Quota */;
      if (vl === 0) s.store.delete(key);
      else s.store.set(key, value);
      if (!s.replaying) {
        const b64 = vl ? base64(value) : null;
        post({ op: "store_set", key, value: b64 });
        s.onStore?.(key, b64);
      }
      return 0;
    }),
    // 5.8 score
    mb_score_submit: ((board, value) => {
      if (!boards.has(board)) return -3 /* NotDeclared */;
      if (!s.replaying) post({ op: "score_submit", board, value: value.toString() });
      return 0;
    }),
    mb_score_show: ((board) => {
      if (boards.has(board) && !s.replaying) post({ op: "score_show", board });
    }),
    // 5.9 links: always through the platform's interstitial.
    mb_link_open: ((ptr, len) => {
      if (len > 2048) return -2 /* InvalidArgument */;
      const url = str(ptr, len);
      if (!/^https?:\/\/[^\s]+$/i.test(url)) return -2 /* InvalidArgument */;
      if (!s.replaying) post({ op: "link_open", url });
      return 0;
    })
  };
  for (const name of HOST_DIRECT) {
    const f = host[name];
    if (typeof f === "function") impl[name] = f;
  }
  const imports = {};
  for (const entry of abi_default.imports) {
    const { name } = entry;
    const req = entry.requires;
    if (!declared(req, m)) {
      imports[name] = (() => {
        throw new Error(`mb.${name} used without declaring ${JSON.stringify(req)} in manifest.toml`);
      });
    } else {
      imports[name] = impl[name] ?? (() => {
        throw new Error(`mb.${name} is not implemented by this host yet`);
      });
    }
  }
  return imports;
}

// src/audio.ts
var MAX_VOICES = 32;
var MASTER_GAIN = 0.8;
var GLIDE = 0.012;
var RESUME_GRACE_MS = 400;
var Audio = class {
  ctx = null;
  master = null;
  decoder = null;
  sounds = [];
  /** Decoded audio by source bytes (shared across sessions via the asset cache). */
  decoded = /* @__PURE__ */ new WeakMap();
  /** Looping voices asked for before their sound finished decoding: they start when it does. */
  waiting = /* @__PURE__ */ new Map();
  voices = /* @__PURE__ */ new Map();
  nextVoice = 1;
  /** The page is active (shown and running), so it may hold a running context. */
  active = false;
  muted = false;
  generation = 0;
  warned = false;
  silence = null;
  constructor() {
    const unlock = () => this.unlock();
    for (const ev of ["pointerdown", "pointerup", "touchend", "click", "keydown"]) {
      window.addEventListener(ev, unlock, { capture: true, passive: true });
    }
  }
  /** WebKit plays Web Audio as "ambient" (muted by the silent switch)
   *  unless the page asks for playback (PLAN Q14). */
  setPlayback(on) {
    const session = navigator.audioSession;
    if (session) {
      session.type = on ? "playback" : "ambient";
      post({ op: "log", level: 0, msg: `runtime: audio session ${session.type}` });
    } else if (on && !this.silence) {
      this.silence = new window.Audio("silence.wav");
      this.silence.loop = true;
      void this.silence.play().catch((e) => post({ op: "log", level: 2, msg: `runtime: silent audio failed: ${e}` }));
      post({ op: "log", level: 0, msg: "runtime: audio session via silent media element" });
    }
  }
  // --- lifecycle ---------------------------------------------------------------
  /** The page is shown and running: create (or revive) the context and start loops. */
  resume() {
    this.active = true;
    if (!this.ctx) {
      this.createContext();
      return;
    }
    const ctx = this.ctx;
    void ctx.resume().catch(() => {
    });
    setTimeout(() => {
      if (this.active && this.ctx === ctx && ctx.state !== "running") this.rebuild(`resume left it ${ctx.state}`);
    }, RESUME_GRACE_MS);
  }
  /** The page is going off screen (or is a neighbour): stop holding audio. */
  suspend() {
    this.active = false;
    void this.ctx?.suspend().catch(() => {
    });
  }
  /** Silences output without stopping voices (a feed neighbour prerolling). */
  setMuted(on) {
    this.muted = on;
    if (this.master) this.master.gain.value = on ? 0 : MASTER_GAIN;
  }
  /** A new game session: the old session's voices stop and sound handles
   *  restart from 1. Decoded audio is kept and reused. */
  newSession() {
    for (const id of [...this.voices.keys()]) this.stop(id);
    this.waiting.clear();
    this.sounds = [];
    this.generation++;
  }
  createContext() {
    const ctx = new AudioContext({ latencyHint: "interactive" });
    const master = ctx.createGain();
    master.gain.value = this.muted ? 0 : MASTER_GAIN;
    const limiter = ctx.createDynamicsCompressor();
    limiter.threshold.value = -9;
    limiter.knee.value = 6;
    limiter.ratio.value = 12;
    limiter.attack.value = 3e-3;
    limiter.release.value = 0.15;
    master.connect(limiter).connect(ctx.destination);
    ctx.onstatechange = () => {
      post({ op: "log", level: 0, msg: `runtime: audio ${ctx.state}` });
      if (ctx.state === "interrupted" && this.active && this.ctx === ctx) {
        setTimeout(() => {
          if (this.active && this.ctx === ctx && ctx.state !== "running") {
            void ctx.resume().catch(() => {
            });
            setTimeout(() => this.active && this.ctx === ctx && ctx.state !== "running" && this.rebuild("interrupted"), RESUME_GRACE_MS);
          }
        }, 100);
      }
    };
    this.ctx = ctx;
    this.master = master;
    post({ op: "log", level: 0, msg: `runtime: audio context created (${ctx.state})` });
    void ctx.resume().catch(() => {
    });
    for (const [id, v] of this.voices) {
      if (v.loop) this.startNodes(id, v);
      else this.voices.delete(id);
    }
    this.flushWaiting();
  }
  /** Replaces a context that won't run with a fresh one. */
  rebuild(why) {
    post({ op: "log", level: 1, msg: `runtime: rebuilding audio (${why})` });
    const old = this.ctx;
    for (const v of this.voices.values()) v.nodes = null;
    this.ctx = null;
    this.master = null;
    void old?.close().catch(() => {
    });
    this.createContext();
  }
  unlock() {
    if (!this.active) return;
    if (!this.ctx) this.createContext();
    const ctx = this.ctx;
    if (ctx.state === "running") return;
    const src = ctx.createBufferSource();
    src.buffer = ctx.createBuffer(1, 1, ctx.sampleRate);
    src.connect(ctx.destination);
    src.start();
    ctx.resume().catch((e) => post({ op: "log", level: 2, msg: `runtime: audio resume failed: ${e}` }));
  }
  // --- sounds and voices -------------------------------------------------------
  /** Returns a sound handle (≥ 1) and decodes in the background. */
  load(bytes) {
    const handle = this.sounds.push(null);
    const gen = this.generation;
    let decoding = this.decoded.get(bytes);
    if (!decoding) {
      this.decoder ??= new OfflineAudioContext({ numberOfChannels: 1, length: 1, sampleRate: 48e3 });
      decoding = this.decoder.decodeAudioData(bytes.slice().buffer);
      this.decoded.set(bytes, decoding);
      decoding.then(
        (buf) => post({ op: "log", level: 0, msg: `runtime: sound decoded (${buf.duration.toFixed(2)} s)` }),
        () => {
        }
      );
    }
    decoding.then((buf) => {
      if (gen !== this.generation) return;
      this.sounds[handle - 1] = buf;
      this.flushWaiting();
    }).catch((e) => post({ op: "log", level: 2, msg: `runtime: sound ${handle} failed to decode: ${e}` }));
    return handle;
  }
  play(sound, vol, pan, pitch, loop) {
    const id = this.nextVoice++;
    if (sound < 1 || sound > this.sounds.length) return id;
    const buf = this.sounds[sound - 1];
    if (!buf || !this.ctx) {
      if (loop) this.waiting.set(id, { sound, vol, pan, pitch });
      return id;
    }
    this.addVoice(id, { buf, vol, pan, pitch, loop, nodes: null });
    return id;
  }
  flushWaiting() {
    if (!this.ctx) return;
    for (const [id, w] of this.waiting) {
      const buf = this.sounds[w.sound - 1];
      if (!buf) continue;
      this.waiting.delete(id);
      this.addVoice(id, { buf, vol: w.vol, pan: w.pan, pitch: w.pitch, loop: true, nodes: null });
    }
  }
  addVoice(id, v) {
    if (this.ctx && this.ctx.state !== "running" && !this.warned) {
      this.warned = true;
      post({ op: "log", level: 2, msg: `runtime: playing while audio is ${this.ctx.state}` });
    }
    if (this.voices.size >= MAX_VOICES) {
      let victim;
      for (const [k, other] of this.voices) {
        if (!other.loop) {
          victim = k;
          break;
        }
      }
      this.stop(victim ?? this.voices.keys().next().value);
    }
    this.voices.set(id, v);
    this.startNodes(id, v);
  }
  startNodes(id, v) {
    if (!this.ctx || !this.master) return;
    const src = this.ctx.createBufferSource();
    src.buffer = v.buf;
    src.loop = v.loop;
    const pan = this.ctx.createStereoPanner();
    const gain = this.ctx.createGain();
    src.connect(pan).connect(gain).connect(this.master);
    v.nodes = { src, pan, gain };
    this.apply(v, true);
    src.onended = () => {
      if (this.voices.get(id) === v && v.nodes?.src === src) this.voices.delete(id);
    };
    src.start();
  }
  update(voice, vol, pan, pitch) {
    const v = this.voices.get(voice);
    if (v) {
      Object.assign(v, { vol, pan, pitch });
      this.apply(v, false);
    }
    const w = this.waiting.get(voice);
    if (w) Object.assign(w, { vol, pan, pitch });
  }
  apply(v, now) {
    if (!v.nodes || !this.ctx) return;
    const t = this.ctx.currentTime;
    const go = (param, value) => {
      if (now) param.value = value;
      else param.setTargetAtTime(value, t, GLIDE);
    };
    go(v.nodes.gain.gain, Math.min(Math.max(v.vol, 0), 4));
    go(v.nodes.pan.pan, Math.min(Math.max(v.pan, -1), 1));
    go(v.nodes.src.playbackRate, Math.min(Math.max(v.pitch, 0.125), 8));
  }
  stop(voice) {
    this.waiting.delete(voice);
    const v = this.voices.get(voice);
    if (!v) return;
    this.voices.delete(voice);
    try {
      v.nodes?.src.stop();
    } catch {
    }
  }
};

// src/boot.ts
async function loadBoot() {
  if (window.__MB_BOOT) return window.__MB_BOOT;
  const res = await fetch("../game/boot.json");
  if (!res.ok) throw new Error(`no boot config (window.__MB_BOOT or game/boot.json): ${res.status}`);
  return await res.json();
}

// src/input.ts
var Input = class {
  constructor(el, layout, touchDeclared, mouseDeclared) {
    this.layout = layout;
    this.touchDeclared = touchDeclared;
    this.mouseDeclared = mouseDeclared;
    el.addEventListener("pointerdown", (e) => this.down(e));
    el.addEventListener("pointermove", (e) => this.move(e));
    el.addEventListener("pointerup", (e) => this.up(e, false));
    el.addEventListener("pointercancel", (e) => this.up(e, true));
    el.style.touchAction = "none";
    el.addEventListener("contextmenu", (e) => e.preventDefault());
  }
  pending = [];
  /** pointerId → small id, for pointers the game owns. */
  owned = /* @__PURE__ */ new Map();
  nextId = 0;
  /** A tap at CSS pixels (the native feed forwards the tap that starts play). */
  tapAt(x, y) {
    const l = this.layout();
    const lx = Math.fround((x - l.offsetX) / l.scale);
    const ly = Math.fround((y - l.offsetY) / l.scale);
    const [down, up] = this.touchDeclared || !this.mouseDeclared ? [1 /* TouchDown */, 3 /* TouchUp */] : [7 /* MouseDown */, 9 /* MouseUp */];
    const id = this.nextId++ & 255;
    this.pending.push([down, id, 0, lx, ly, 1, 0, 0], [up, id, 0, lx, ly, 0, 0, 0]);
  }
  /** Queues an event from another source (keyboard, gamepad). */
  push(e) {
    this.pending.push(e);
  }
  /** Moves queued events to the guest's frame. Call once per frame boundary. */
  take() {
    const out = this.pending;
    this.pending = [];
    return out;
  }
  asTouch(e) {
    return e.pointerType !== "mouse" || !this.mouseDeclared;
  }
  pointer(kind, id, e) {
    const l = this.layout();
    const x = Math.fround((e.clientX - l.offsetX) / l.scale);
    const y = Math.fround((e.clientY - l.offsetY) / l.scale);
    this.pending.push([kind, id, 0, x, y, Math.fround(e.pressure), 0, 0]);
  }
  down(e) {
    const l = this.layout();
    if (e.clientY >= l.height - l.stripHeight) return;
    const touch = this.asTouch(e);
    if (touch ? !this.touchDeclared : !this.mouseDeclared) return;
    const id = touch ? this.nextId++ & 255 : e.button & 255;
    this.owned.set(e.pointerId, id);
    e.target.setPointerCapture?.(e.pointerId);
    this.pointer(touch ? 1 /* TouchDown */ : 7 /* MouseDown */, id, e);
  }
  move(e) {
    const id = this.owned.get(e.pointerId);
    if (id === void 0) {
      if (e.pointerType === "mouse" && this.mouseDeclared) this.pointer(8 /* MouseMove */, 0, e);
      return;
    }
    this.pointer(this.asTouch(e) ? 2 /* TouchMove */ : 8 /* MouseMove */, id, e);
  }
  up(e, cancel) {
    const id = this.owned.get(e.pointerId);
    if (id === void 0) return;
    this.owned.delete(e.pointerId);
    const kind = this.asTouch(e) ? cancel ? 4 /* TouchCancel */ : 3 /* TouchUp */ : 9 /* MouseUp */;
    this.pointer(kind, id, e);
  }
};
function writeEvents(view, ptr, events, time) {
  events.forEach((ev, i) => {
    const o = ptr + i * 32;
    view.setUint8(o, ev[0]);
    view.setUint8(o + 1, ev[1]);
    view.setUint16(o + 2, ev[2], true);
    view.setFloat32(o + 4, ev[3], true);
    view.setFloat32(o + 8, ev[4], true);
    view.setFloat32(o + 12, ev[5], true);
    view.setFloat32(o + 16, ev[6], true);
    view.setUint32(o + 20, ev[7], true);
    view.setFloat64(o + 24, time, true);
  });
}

// src/motion.ts
var q = (v, steps, lo, hi) => Math.round(Math.max(lo, Math.min(hi, v)) * steps) / steps;
var vec = (x, y, z, max) => [x, y, z].every(Number.isFinite) ? [q(x, 4096, -max, max), q(y, 4096, -max, max), q(z, 4096, -max, max)] : null;
var Sensors = class {
  constructor(want, emulate) {
    this.want = want;
    if (emulate) this.emulate();
  }
  /** Latest samples; null until the first one arrives. */
  tilt = null;
  motion = null;
  /** 0–1, or −1 when the mic is unavailable (denied, missing). */
  loudness = null;
  /** Native: gravity and user acceleration, game axes, in g. */
  set(gx, gy, gz, ax, ay, az) {
    if (this.want.tilt) this.tilt = vec(gx, gy, gz, 4) ?? this.tilt;
    if (this.want.motion) this.motion = vec(ax, ay, az, 16) ?? this.motion;
  }
  setLoudness(v) {
    if (!this.want.loudness || !Number.isFinite(v)) return;
    this.loudness = v < 0 ? -1 : q(v, 1024, 0, 1);
  }
  /** Preview: a console-pinned loudness that the emulation leaves alone (undefined unpins). */
  pinned;
  /** `mb.loudness()` from outside: native samples, or a preview pin. */
  external(v) {
    if (this.emulated) {
      this.pinned = v;
      if (v !== void 0) this.setLoudness(v);
    } else if (v !== void 0) {
      this.setLoudness(v);
    }
  }
  emulated = false;
  emulate() {
    this.emulated = true;
    if (this.want.tilt) this.tilt = [0, 1, 0];
    if (this.want.motion) this.motion = [0, 0, 0];
    let last = null;
    window.addEventListener("pointermove", (e) => {
      const angle = (pos, size) => Math.max(-1, Math.min(1, (pos / size - 0.5) * 3)) * (Math.PI / 2);
      const roll = angle(e.clientX, window.innerWidth);
      const pitch = angle(e.clientY, window.innerHeight);
      if (this.want.tilt) this.tilt = vec(Math.sin(roll) * Math.cos(pitch), Math.cos(roll) * Math.cos(pitch), -Math.sin(pitch), 4);
      const t = e.timeStamp / 1e3;
      if (last && t > last.t) {
        const dt = t - last.t;
        const vx = (e.clientX - last.x) / dt;
        const vy = (e.clientY - last.y) / dt;
        const k = 6e-3;
        if (this.want.motion) this.motion = vec((vx - last.vx) * k, (vy - last.vy) * k, 0, 16);
        last = { x: e.clientX, y: e.clientY, vx, vy, t };
      } else {
        last = { x: e.clientX, y: e.clientY, vx: 0, vy: 0, t };
      }
    });
    if (this.want.motion) {
      setInterval(() => {
        if (this.motion) this.motion = vec(this.motion[0] * 0.5, this.motion[1] * 0.5, this.motion[2] * 0.5, 16);
      }, 50);
    }
    if (this.want.loudness) this.emulateMic();
  }
  /** Desktop preview: the real microphone if the browser allows it, else keys (see top). */
  emulateMic() {
    this.loudness = 0;
    let held = null;
    let mic = null;
    let level = 0;
    const keyLevel = (e) => e.code === "Space" ? 0.85 : /^Digit[0-9]$/.test(e.code) ? Number(e.code.slice(5)) / 10 : null;
    window.addEventListener("keydown", (e) => {
      const k = keyLevel(e);
      if (k !== null) held = k;
    });
    window.addEventListener("keyup", (e) => {
      if (keyLevel(e) !== null) held = null;
    });
    const start = async () => {
      window.removeEventListener("pointerdown", start);
      try {
        const stream = await navigator.mediaDevices.getUserMedia({ audio: { echoCancellation: false, autoGainControl: false, noiseSuppression: false } });
        const ctx = new AudioContext();
        mic = ctx.createAnalyser();
        mic.fftSize = 1024;
        ctx.createMediaStreamSource(stream).connect(mic);
        console.log("[mb] preview: loudness from the microphone (or hold 1\u20139 / Space)");
      } catch {
        console.log("[mb] preview: no microphone; hold 1\u20139 for 0.1\u20130.9, Space for a blow (0.85)");
      }
    };
    window.addEventListener("pointerdown", start);
    const buf = new Float32Array(1024);
    setInterval(() => {
      if (this.pinned !== void 0) return;
      let target = held ?? 0;
      if (mic) target = Math.max(target, levelOf(mic, buf));
      level += (target - level) * (target > level ? 0.6 : 0.15);
      this.setLoudness(level);
    }, 16);
  }
};
function levelOf(a, buf) {
  a.getFloatTimeDomainData(buf);
  let s = 0;
  for (const v of buf) s += v * v;
  const db = 10 * Math.log10(s / buf.length + 1e-12);
  return Math.max(0, Math.min(1, (db + 60) / 60));
}

// src/keys.ts
var HID = {
  Enter: 40,
  Escape: 41,
  Backspace: 42,
  Tab: 43,
  Space: 44,
  Minus: 45,
  Equal: 46,
  BracketLeft: 47,
  BracketRight: 48,
  Backslash: 49,
  Semicolon: 51,
  Quote: 52,
  Backquote: 53,
  Comma: 54,
  Period: 55,
  Slash: 56,
  CapsLock: 57,
  ArrowRight: 79,
  ArrowLeft: 80,
  ArrowDown: 81,
  ArrowUp: 82,
  ControlLeft: 224,
  ShiftLeft: 225,
  AltLeft: 226,
  MetaLeft: 227,
  ControlRight: 228,
  ShiftRight: 229,
  AltRight: 230,
  MetaRight: 231
};
for (let i = 0; i < 26; i++) HID[`Key${String.fromCharCode(65 + i)}`] = 4 + i;
for (let i = 1; i <= 9; i++) HID[`Digit${i}`] = 29 + i;
HID.Digit0 = 39;
for (let i = 1; i <= 12; i++) HID[`F${i}`] = 57 + i;
var KEY_DOWN = 5;
var KEY_UP = 6;
var PAD_BUTTON = 11;
var PAD_AXIS = 12;
var Keyboard = class {
  constructor(push) {
    const on = (kind) => (e) => {
      const code = HID[e.code];
      if (code === void 0 || e.repeat) return;
      e.preventDefault();
      push([kind, 0, 0, 0, 0, 0, 0, code]);
    };
    window.addEventListener("keydown", on(KEY_DOWN));
    window.addEventListener("keyup", on(KEY_UP));
  }
};
var Gamepads = class {
  last = /* @__PURE__ */ new Map();
  poll(push) {
    for (const pad of navigator.getGamepads?.() ?? []) {
      if (!pad) continue;
      const prev = this.last.get(pad.index) ?? { buttons: [], axes: [] };
      const buttons = pad.buttons.map((b) => Math.fround(b.value));
      const axes = pad.axes.map((a) => Math.fround(Math.abs(a) < 0.1 ? 0 : a));
      buttons.forEach((v, i) => {
        if (v > 0.5 !== (prev.buttons[i] ?? 0) > 0.5) push([PAD_BUTTON, 0, pad.index, 0, 0, v, 0, i]);
      });
      axes.forEach((v, i) => {
        if (Math.abs(v - (prev.axes[i] ?? 0)) > 0.02) push([PAD_AXIS, 0, pad.index, 0, 0, v, 0, i]);
      });
      this.last.set(pad.index, { buttons, axes });
    }
  }
};

// src/probe.ts
async function probe() {
  const out = {
    userAgent: navigator.userAgent,
    webgpu: "gpu" in navigator,
    offscreenCanvas: typeof OffscreenCanvas !== "undefined",
    transferControlToOffscreen: "transferControlToOffscreen" in HTMLCanvasElement.prototype,
    crossOriginIsolated: self.crossOriginIsolated
  };
  try {
    out.worker = await new Promise((resolve) => {
      const w = new Worker(new URL("probe-worker.js", import.meta.url), { type: "module" });
      const timer = setTimeout(() => resolve({ error: "timeout" }), 3e3);
      w.onmessage = (e) => {
        clearTimeout(timer);
        resolve(e.data);
        w.terminate();
      };
      w.onerror = (e) => {
        clearTimeout(timer);
        resolve({ error: e.message || "worker failed to start" });
      };
      const c = new OffscreenCanvas(16, 16);
      w.postMessage({ canvas: c }, [c]);
    });
  } catch (e) {
    out.worker = { error: String(e) };
  }
  return out;
}

// src/replay.ts
var Recorder = class {
  log;
  constructor(header) {
    this.log = { version: 0, ...header, frames: [] };
  }
  frame(dt, events, assets, sensors) {
    const f = { dt, ...sensors };
    if (events.length) f.events = events;
    if (assets.length) f.assets = assets;
    this.log.frames.push(f);
  }
};

// src/session.ts
var Session = class {
  constructor(audio, seed, dailySeed, playerId, locale, screen, gameBase, store, assetCache) {
    this.audio = audio;
    this.seed = seed;
    this.dailySeed = dailySeed;
    this.playerId = playerId;
    this.locale = locale;
    this.screen = screen;
    this.assets = new Assets(gameBase, assetCache);
    for (const [k, v] of Object.entries(store)) this.store.set(k, Uint8Array.from(atob(v), (c) => c.charCodeAt(0)));
  }
  gameTime = 0;
  frameEvents = [];
  cursor = 0;
  memory;
  assets;
  /** Saved data: in-page copy, written through to native (SPEC §5.8). */
  store = /* @__PURE__ */ new Map();
  /** Replays never persist or submit anything. */
  replaying = false;
  /** Called when saved data changes (key, base64 value or null). */
  onStore = null;
  /** Last SPEC §5.1 round state the game reported (0 idle, 1 playing, 2 over). */
  round = 0;
  /** This frame's sensor snapshot (SPEC §5.7); null before the first sample. */
  tilt = null;
  motion = null;
  loudness = null;
  /** Total bytes of keys + values, for the 256 KB quota. */
  storeBytes() {
    let n = 0;
    for (const [k, v] of this.store) n += k.length + v.length;
    return n;
  }
  /** Starts a frame: delivers its events and sensor snapshot, advances game time by dt. */
  beginFrame(dt, events, sensors = {}) {
    this.gameTime += dt;
    if (sensors.tilt) this.tilt = sensors.tilt;
    if (sensors.motion) this.motion = sensors.motion;
    if (sensors.loud !== void 0) this.loudness = sensors.loud;
    this.frameEvents = events;
    this.cursor = 0;
  }
  pollInput(view, ptr, cap) {
    const n = Math.max(0, Math.min(cap, this.frameEvents.length - this.cursor));
    writeEvents(view, ptr, this.frameEvents.slice(this.cursor, this.cursor + n), this.gameTime);
    this.cursor += n;
    return n;
  }
  log(level, msg) {
    const fn = level >= 3 ? console.error : level === 2 ? console.warn : console.log;
    fn(`[game] ${msg}`);
    post({ op: "log", level, msg: msg.slice(0, 2e3) });
  }
};
var Assets = class {
  constructor(base, cache) {
    this.base = base;
    this.cache = cache;
  }
  list = [];
  load(path) {
    if (!/^[A-Za-z0-9_./-]+$/.test(path) || path.split("/").some((c) => c === "" || c === "." || c === "..")) {
      return -2;
    }
    const a = { path, state: 0, fetching: Promise.resolve() };
    let bytes = this.cache.get(path);
    if (!bytes) {
      bytes = fetch(`${this.base}${path}`).then(async (r) => r.ok ? { state: 1, data: new Uint8Array(await r.arrayBuffer()) } : { state: -1 }).catch(() => ({ state: -1 }));
      this.cache.set(path, bytes);
    }
    a.fetching = bytes.then((r) => {
      a.arrived = r;
    });
    this.list.push(a);
    return this.list.length;
  }
  get(h) {
    return this.list[h - 1];
  }
  state(h) {
    return this.get(h)?.state ?? -1;
  }
  data(h) {
    return this.get(h)?.data;
  }
  /** Every asset requested so far has arrived (fetched or failed). */
  allArrived() {
    return this.list.every((a) => a.state !== 0 || a.arrived !== void 0);
  }
  /** Live play: publishes everything that has arrived; returns the handles. */
  publishArrived() {
    const out = [];
    this.list.forEach((a, i) => {
      if (a.state === 0 && a.arrived) {
        this.publish(a);
        out.push(i + 1);
      }
    });
    return out;
  }
  /** Real-time replay: publishes the recorded handles if they've all arrived. */
  tryPublishRecorded(handles) {
    const assets = handles.map((h) => this.get(h));
    if (assets.some((a) => !a || a.state === 0 && !a.arrived)) return false;
    for (const a of assets) if (a && a.state === 0) this.publish(a);
    return true;
  }
  /** Replay: publishes exactly the recorded handles, waiting for their fetches. */
  async publishRecorded(handles) {
    for (const h of handles) {
      const a = this.get(h);
      if (!a) throw new Error(`replay references unknown asset handle ${h}`);
      await a.fetching;
      this.publish(a);
    }
  }
  publish(a) {
    a.state = a.arrived?.state ?? -1;
    a.data = a.arrived?.data;
  }
};

// src/main.ts
var INIT_BUDGET_MS = 500;
var CALLBACK_BUDGET_MS = 250;
var HEARTBEAT_EVERY = 15;
var FUEL_PER_CALLBACK = 3e8;
var GAME = "../game/";
var PREROLL_MIN_FRAMES = 6;
var PREROLL_MAX_MS = 3e3;
var Runtime = class _Runtime {
  constructor(guest, session, input, recorder, gamepads, sensors, source) {
    this.guest = guest;
    this.session = session;
    this.input = input;
    this.recorder = recorder;
    this.gamepads = gamepads;
    this.sensors = sensors;
    this.source = source;
  }
  running = false;
  suspended = false;
  /** False while a feed neighbour waits for its first resume. */
  started = true;
  last = null;
  frames = 0;
  stopped = false;
  raf = 0;
  /** Next frame of a replay being played back. */
  cursor = 0;
  /** Sensor readings last handed to the guest (recorded only when they change). */
  seen = { tilt: "", motion: "", loud: null };
  onReplayEnd = null;
  /** Preview only (SPEC §9): multiplies live dt, for watching fast moments slowly. */
  static timeScale = 1;
  start() {
    this.running = true;
    this.session.audio.resume();
    this.raf = requestAnimationFrame((t) => this.tick(t));
  }
  /** Feed neighbour: run silently until the assets it asked for have loaded
   *  (so the waiting card shows the real game, not its loading fallback),
   *  then hold that frame until `resume()`. */
  startPaused() {
    this.running = true;
    this.suspended = true;
    this.started = false;
    this.prerolling = true;
    this.session.audio.setMuted(true);
    const deadline = performance.now() + PREROLL_MAX_MS;
    let n = 0;
    const frame = () => {
      if (!this.prerolling || this.stopped) return;
      this.drawing = false;
      if (this.source.kind === "replay") this.replayTick();
      else this.liveFrame(Math.fround(1 / 60));
      this.drawing = true;
      n++;
      const loaded = this.session.assets.allArrived() && n >= PREROLL_MIN_FRAMES;
      if (loaded || performance.now() > deadline) {
        this.prerolling = false;
        this.session.audio.setMuted(false);
        this.draw();
        return;
      }
      this.raf = requestAnimationFrame(frame);
    };
    frame();
  }
  /** Running silently ahead of being shown (startPaused). */
  prerolling = false;
  /** False while prerolling: frames update without drawing. */
  drawing = true;
  suspend() {
    if (this.suspended || this.stopped) return;
    this.suspended = true;
    cancelAnimationFrame(this.raf);
    this.session.audio.suspend();
    this.timed("mb_suspend", () => this.guest.mb_suspend?.());
    post({ op: "suspended" });
  }
  resume() {
    if (!this.suspended || this.stopped) return;
    if (this.prerolling) {
      this.prerolling = false;
      cancelAnimationFrame(this.raf);
      this.session.audio.setMuted(false);
    }
    this.suspended = false;
    this.last = null;
    this.input.take();
    this.session.audio.resume();
    if (this.started) this.timed("mb_resume", () => this.guest.mb_resume?.());
    this.started = true;
    post({ op: "resumed" });
    this.raf = requestAnimationFrame((t) => this.tick(t));
  }
  tick(now) {
    if (!this.running || this.suspended || this.stopped) return;
    if (this.source.kind === "replay") {
      this.replayTick();
    } else {
      const raw = this.last === null ? 1 / 60 : Math.max((now - this.last) / 1e3, 0);
      this.last = now;
      this.liveFrame(Math.fround(Math.min(raw * _Runtime.timeScale, 0.1)));
    }
    if (++this.frames % HEARTBEAT_EVERY === 0) {
      post({ op: "heartbeat", frame: this.frames, time: this.session.gameTime });
    }
    if (!this.stopped) this.raf = requestAnimationFrame((t) => this.tick(t));
  }
  liveFrame(dt) {
    this.gamepads?.poll((e) => this.input.push(e));
    const events = this.input.take();
    const assets = this.session.assets.publishArrived();
    const sensors = this.sampleSensors();
    this.recorder.frame(dt, events, assets, sensors);
    this.step(dt, events, sensors);
  }
  /** Preview only: advance a suspended live session by `n` frames of 1/60 s. */
  stepFrames(n) {
    if (!this.suspended || this.stopped || this.source.kind !== "live") return;
    for (let i = 0; i < n && !this.stopped; i++) this.liveFrame(Math.fround(1 / 60));
  }
  /** The sensor readings that differ from what the guest saw last frame. */
  sampleSensors() {
    const s = this.sensors;
    const d = {};
    if (!s) return d;
    if (s.tilt && s.tilt.join() !== this.seen.tilt) {
      this.seen.tilt = s.tilt.join();
      d.tilt = [...s.tilt];
    }
    if (s.motion && s.motion.join() !== this.seen.motion) {
      this.seen.motion = s.motion.join();
      d.motion = [...s.motion];
    }
    if (s.loudness !== null && s.loudness !== this.seen.loud) {
      this.seen.loud = s.loudness;
      d.loud = s.loudness;
    }
    return d;
  }
  /** Plays one recorded frame per display frame; live input is ignored. */
  replayTick() {
    if (this.source.kind !== "replay") return;
    this.input.take();
    const f = this.source.log.frames[this.cursor];
    if (!f) {
      this.onReplayEnd?.();
      return;
    }
    if (!this.session.assets.tryPublishRecorded(f.assets ?? [])) return;
    this.cursor++;
    this.step(f.dt, f.events ?? [], f);
  }
  /** One frame: deliver input, update, render, present. */
  step(dt, events, sensors) {
    this.session.beginFrame(dt, events, sensors);
    this.timed("mb_update", () => this.guest.mb_update(dt));
    if (this.stopped || !this.drawing) return;
    this.draw();
  }
  draw() {
    host_begin_frame();
    this.timed("mb_render", () => this.guest.mb_render());
    if (this.stopped) return;
    host_end_frame();
  }
  timed(name, f, budget = CALLBACK_BUDGET_MS) {
    this.guest.__mb_fuel.value = FUEL_PER_CALLBACK;
    const t = performance.now();
    try {
      f();
    } catch (e) {
      this.stop();
      if (this.guest.__mb_fuel.value === 0) {
        post({ op: "watchdog", callback: `${name} (loop fuel exhausted)`, ms: performance.now() - t });
      } else {
        post({ op: "error", message: `${name} trapped: ${e instanceof Error ? e.message : String(e)}` });
      }
      return;
    }
    const ms = performance.now() - t;
    if (ms > budget) {
      this.stop();
      post({ op: "watchdog", callback: name, ms });
    }
  }
  stop() {
    this.stopped = true;
    cancelAnimationFrame(this.raf);
  }
  snapshot() {
    return { frames: this.frames, time: this.session.gameTime, hash: host_draw_hash(), log: structuredClone(this.recorder.log) };
  }
};
function hexBytes(hex) {
  const out = new Uint8Array(16);
  for (let i = 0; i < 16; i++) out[i] = parseInt(hex.slice(i * 2, i * 2 + 2), 16) || 0;
  return out;
}
function fit(canvas, boot) {
  const [lw, lh] = boot.manifest.logical_size;
  const cssW = window.innerWidth;
  const cssH = window.innerHeight;
  const scale = Math.min(cssW / lw, cssH / lh) || 1;
  const offsetX = (cssW - lw * scale) / 2;
  const offsetY = (cssH - lh * scale) / 2;
  const k = Math.min(window.devicePixelRatio || 1, 2 / scale);
  const size = [Math.max(1, Math.round(cssW * k)), Math.max(1, Math.round(cssH * k))];
  canvas.width = size[0];
  canvas.height = size[1];
  const ins = boot.insets;
  const toLogical = (css, off) => Math.max(0, css - off) / scale;
  const screen = new Float32Array([
    lw,
    lh,
    toLogical(ins.top, offsetY),
    toLogical(ins.right, offsetX),
    toLogical(Math.max(ins.bottom, boot.stripHeight), offsetY),
    toLogical(ins.left, offsetX),
    k * scale,
    0
  ]);
  return { layout: { scale, offsetX, offsetY, height: cssH, stripHeight: boot.stripHeight }, screen, size };
}
function drawOverlays(layout, screen, mic) {
  const root = document.createElement("div");
  root.style.cssText = "position:fixed;inset:0;pointer-events:none;z-index:10";
  document.body.append(root);
  const draw = () => {
    const l = layout();
    const lw = screen[0] ?? 360, lh = screen[1] ?? 640, insetTop = screen[2] ?? 0;
    const box = (x, y, w, h, label) => `<div style="position:absolute;left:${l.offsetX + x * l.scale}px;top:${l.offsetY + y * l.scale}px;width:${w * l.scale}px;height:${h * l.scale}px;background:rgba(255,0,128,.25);outline:1px dashed #f08;color:#fff;font:10px system-ui;padding:2px;box-sizing:border-box">${label}</div>`;
    root.innerHTML = box(0, lh - 70, lw, 70, "card: title + creator") + box(lw - 56, lh * 0.45, 56, lh * 0.55 - 70, "card: buttons") + box(8, insetTop + 4, mic ? 110 : 64, 44, mic ? "pause pill + mic badge (play)" : "pause pill (play)");
  };
  draw();
  return draw;
}
async function createSession(page, source, seedOverride) {
  const { boot, manifest: m } = page;
  const log = source.kind === "replay" ? source.log : void 0;
  const seed = BigInt(log?.seed ?? seedOverride ?? boot.seed);
  const dailySeed = BigInt(log?.dailySeed ?? boot.dailySeed);
  const store = log?.store ?? { ...page.store };
  const playerId = log?.playerId ?? boot.playerId;
  const locale = log?.locale ?? boot.locale;
  const screen = log ? new Float32Array(log.screen) : page.screen;
  host_new_session();
  page.audio.newSession();
  const session = new Session(page.audio, seed, dailySeed, hexBytes(playerId), locale, screen, GAME, store, page.assets);
  session.replaying = source.kind === "replay";
  session.onStore = (key, value) => {
    if (value === null) delete page.store[key];
    else page.store[key] = value;
  };
  const instance = await WebAssembly.instantiate(page.module, { mb: buildImports(session, page.host, m) });
  const guest = instance.exports;
  if (!(guest.__mb_fuel instanceof WebAssembly.Global)) throw new Error("game.wasm is not loop-metered; refusing to run it");
  session.memory = () => guest.memory;
  const recorder = new Recorder({
    game: m.id,
    seed: seed.toString(),
    dailySeed: dailySeed.toString(),
    store,
    playerId,
    locale,
    screen: Array.from(screen)
  });
  const rt = new Runtime(guest, session, page.input, recorder, page.gamepads, page.sensors, source);
  rt.timed("mb_init", () => guest.mb_init(), INIT_BUDGET_MS);
  return rt;
}
async function main() {
  const t0 = performance.now();
  window.addEventListener("error", (e) => post({ op: "error", message: String(e.message) }));
  window.addEventListener("unhandledrejection", (e) => post({ op: "error", message: String(e.reason) }));
  const boot = await loadBoot();
  const m = boot.manifest;
  const canvas = document.getElementById("game");
  let { layout, screen } = fit(canvas, boot);
  const host = await __wbg_init({ module_or_path: new URL("mb_host_bg.wasm", import.meta.url) });
  await host_init(canvas, m.logical_size[0], m.logical_size[1]);
  const module = await WebAssembly.compile(await (await fetch(`${GAME}game.wasm`)).arrayBuffer());
  const input = new Input(canvas, () => layout, m.inputs.includes("touch"), m.inputs.includes("mouse"));
  if (m.inputs.includes("keyboard")) new Keyboard((e) => input.push(e));
  const page = {
    boot,
    manifest: m,
    module,
    host,
    audio: new Audio(),
    input,
    gamepads: m.inputs.includes("gamepad") ? new Gamepads() : null,
    sensors: ["tilt", "motion", "loudness"].some((x) => m.sensors.includes(x)) ? new Sensors(
      { tilt: m.sensors.includes("tilt"), motion: m.sensors.includes("motion"), loudness: m.sensors.includes("loudness") },
      !hasNative
    ) : null,
    screen,
    store: { ...boot.store ?? {} },
    assets: /* @__PURE__ */ new Map()
  };
  const refit = () => {
    const f = fit(canvas, boot);
    layout = f.layout;
    host_resize(f.size[0], f.size[1]);
  };
  window.addEventListener("resize", refit);
  let rt;
  const restart = async (opts) => {
    rt?.stop();
    const source = opts.replay ? { kind: "replay", log: opts.replay, loop: opts.loop ?? true } : { kind: "live" };
    const next = await createSession(page, source, opts.seed);
    if (source.kind === "replay") {
      next.onReplayEnd = () => {
        if (source.loop) void restart({ ...opts, paused: false });
        else next.suspend();
      };
    }
    rt = next;
    if (opts.paused) next.startPaused();
    else next.start();
    post({ op: "restarted", replay: source.kind === "replay" });
  };
  page.audio.setPlayback(boot.soundOn === true);
  window.mb = {
    sound: (on) => page.audio.setPlayback(on),
    suspend: () => rt.suspend(),
    resume: () => rt.resume(),
    snapshot: () => rt.snapshot(),
    resize: refit,
    restart: (json) => restart(JSON.parse(json)),
    tap: (x, y) => input.tapAt(x, y),
    sensors: (gx, gy, gz, ax, ay, az) => page.sensors?.set(gx, gy, gz, ax, ay, az),
    loudness: (v) => page.sensors?.external(v),
    step: (n = 1) => rt.stepFrames(n),
    speed: (k) => {
      Runtime.timeScale = Math.max(0.01, Math.min(4, k));
    }
  };
  if (!hasNative) {
    const q2 = new URLSearchParams(location.search);
    if (q2.has("speed")) window.mb.speed(Number(q2.get("speed")) || 1);
    if (q2.has("overlays")) {
      const draw = drawOverlays(() => layout, screen, m.sensors.includes("loudness"));
      window.addEventListener("resize", () => requestAnimationFrame(draw));
    }
  }
  if (boot.replay) {
    rt = await createSession(page, { kind: "replay", log: boot.replay, loop: false });
    post({ op: "ready", info: JSON.parse(host_info()), bootMs: performance.now() - t0, paused: false });
    for (const f of boot.replay.frames) {
      await rt.session.assets.publishRecorded(f.assets ?? []);
      rt.step(f.dt, f.events ?? [], f);
    }
    post({ op: "replay-done", frames: boot.replay.frames.length, hash: host_draw_hash() });
    return;
  }
  rt = await createSession(page, { kind: "live" });
  const paused = boot.startPaused === true;
  post({ op: "ready", info: JSON.parse(host_info()), bootMs: performance.now() - t0, paused });
  void probe().then((p) => post({ op: "probe", probe: p }));
  if (paused) rt.startPaused();
  else rt.start();
}
main().catch((e) => post({ op: "error", message: `boot failed: ${e instanceof Error ? e.stack ?? e.message : String(e)}` }));
