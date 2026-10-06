package com.openforum.controller;

import com.openforum.common.ApiResponse;
import com.openforum.dto.plugin.PluginDTO;
import com.openforum.dto.plugin.PluginToggleRequest;
import com.openforum.security.SecurityUtils;
import com.openforum.service.PluginService;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.util.List;

@RestController
@RequestMapping("/plugins")
public class PluginController {

    private final PluginService pluginService;

    public PluginController(PluginService pluginService) {
        this.pluginService = pluginService;
    }

    @GetMapping
    public ApiResponse<List<PluginDTO>> list() {
        return ApiResponse.ok(pluginService.list());
    }

    @PutMapping("/{key}/enabled")
    public ApiResponse<PluginDTO> setEnabled(@PathVariable String key, @RequestBody PluginToggleRequest request) {
        SecurityUtils.requireAdmin();
        return ApiResponse.ok(pluginService.setEnabled(key, request.enabled()));
    }
}
