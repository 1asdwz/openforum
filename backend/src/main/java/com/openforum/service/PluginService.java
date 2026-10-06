package com.openforum.service;

import com.openforum.dto.plugin.PluginDTO;
import com.openforum.exception.ApiException;
import com.openforum.plugin.ForumPlugin;
import com.openforum.plugin.PluginRegistry;
import org.springframework.stereotype.Service;

import java.util.List;

@Service
public class PluginService {

    private final PluginRegistry pluginRegistry;

    public PluginService(PluginRegistry pluginRegistry) {
        this.pluginRegistry = pluginRegistry;
    }

    public List<PluginDTO> list() {
        return pluginRegistry.all().stream().map(this::toDto).toList();
    }

    public PluginDTO setEnabled(String key, boolean enabled) {
        pluginRegistry.setEnabled(key, enabled);
        ForumPlugin plugin = pluginRegistry.byKey(key)
                .orElseThrow(() -> ApiException.notFound("插件不存在: " + key));
        return toDto(plugin);
    }

    private PluginDTO toDto(ForumPlugin plugin) {
        return new PluginDTO(plugin.getKey(), plugin.getName(), plugin.getType().name(),
                pluginRegistry.isEnabled(plugin), plugin.getDescription());
    }
}
