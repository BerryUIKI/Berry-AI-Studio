import { invoke } from "@tauri-apps/api/core";
import type { ImageFile } from "../types";

export interface ComfyResponse {
  prompt_id?: string;
  number?: number;
  node_errors?: Record<string, any>;
  error?: string;
}

export interface WebUIResponse {
  images?: string[];
  parameters?: Record<string, any>;
  info?: string;
}

export interface ServiceStatus {
  online: boolean;
  checking: boolean;
  error?: string;
}

/**
 * Check connectivity of ComfyUI or SD WebUI instance.
 */
export async function checkServiceStatus(
  endpoint: string,
  serviceType: "comfyui" | "webui",
): Promise<boolean> {
  if (!endpoint || !endpoint.trim()) return false;
  try {
    return await invoke<boolean>("check_generation_service", {
      endpoint: endpoint.trim(),
      serviceType,
    });
  } catch (err) {
    console.warn(`Failed to check ${serviceType} service status:`, err);
    return false;
  }
}

/**
 * Extract raw workflow JSON from an ImageFile if present.
 */
export function extractWorkflowJson(file: ImageFile | null): string | null {
  if (!file?.metadata) return null;
  const raw = file.metadata.raw?.trim();
  if (raw && (raw.startsWith("{") || raw.startsWith("["))) {
    try {
      JSON.parse(raw);
      return raw;
    } catch {
      // not valid JSON
    }
  }

  const params = file.metadata.parameters?.trim();
  if (params && (params.startsWith("{") || params.startsWith("["))) {
    try {
      JSON.parse(params);
      return params;
    } catch {
      // not valid JSON
    }
  }

  return null;
}

/**
 * Check whether the file has a ComfyUI workflow or prompt graph.
 */
export function hasComfyWorkflow(file: ImageFile | null): boolean {
  if (!file?.metadata) return false;
  if (file.metadata.format === "ComfyUi") return true;
  return extractWorkflowJson(file) !== null;
}

/**
 * Check whether the file has prompt metadata that can be sent to SD WebUI.
 */
export function hasPromptData(file: ImageFile | null): boolean {
  if (!file?.metadata) return false;
  return !!(file.metadata.prompt && file.metadata.prompt.trim());
}

/**
 * Send workflow JSON to ComfyUI via backend proxy.
 */
export async function sendWorkflowToComfyUI(
  endpoint: string,
  workflowJson: string,
): Promise<ComfyResponse> {
  return await invoke<ComfyResponse>("send_to_comfyui", {
    endpoint: endpoint.trim(),
    workflowJson,
  });
}

/**
 * Send generation parameters to SD WebUI via backend proxy.
 */
export async function sendPromptToWebUI(
  endpoint: string,
  file: ImageFile,
): Promise<WebUIResponse> {
  const meta = file.metadata;
  const payload: Record<string, any> = {
    prompt: meta?.prompt || "",
    negative_prompt: meta?.negative_prompt || "",
    steps: meta?.steps ?? 20,
    cfg_scale: meta?.cfg_scale ?? 7.0,
    sampler_name: meta?.sampler || "Euler a",
    width: meta?.width ?? 512,
    height: meta?.height ?? 512,
  };

  if (meta?.seed) {
    const parsedSeed = parseInt(meta.seed, 10);
    payload.seed = isNaN(parsedSeed) ? -1 : parsedSeed;
  } else {
    payload.seed = -1;
  }

  return await invoke<WebUIResponse>("send_to_webui", {
    endpoint: endpoint.trim(),
    payload,
  });
}
