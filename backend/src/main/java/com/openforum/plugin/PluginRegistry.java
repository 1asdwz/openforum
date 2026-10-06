package com.openforum.plugin;

import com.openforum.exception.ApiException;
import org.springframework.stereotype.Component;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.ConcurrentHashMap;

@Component
public class PluginRegistry {

    private final Map<String, ForumPlugin> plugins = new LinkedHashMap<>();
    private final Map<String, Boolean> overrides = new ConcurrentHashMap<>();

    public PluginRegistry(List<ForumPlugin> discovered) {
        List<ForumPlugin> sorted = new ArrayList<>(discovered);
        sorted.sort(Comparator.comparing(ForumPlugin::getKey));
        for (ForumPlugin plugin : sorted) {
            plugins.put(plugin.getKey(), plugin);
        }
    }

    public List<ForumPlugin> all() {
        return List.copyOf(plugins.values());
    }

    public Optional<ForumPlugin> byKey(String key) {
        return Optional.ofNullable(plugins.get(key));
    }

    public boolean isEnabled(ForumPlugin plugin) {
        return overrides.getOrDefault(plugin.getKey(), plugin.isEnabledByDefault());
    }

    public List<ReviewPlugin> enabledReviewPlugins() {
        List<ReviewPlugin> result = new ArrayList<>();
        for (ForumPlugin plugin : plugins.values()) {
            if (plugin.getType() == PluginType.REVIEW
                    && plugin instanceof ReviewPlugin reviewPlugin
                    && isEnabled(plugin)
                    && reviewPlugin.isAvailable()) {
                result.add(reviewPlugin);
            }
        }
        return result;
    }

    public List<DocumentReviewPlugin> enabledDocumentReviewPlugins() {
        List<DocumentReviewPlugin> result = new ArrayList<>();
        for (ForumPlugin plugin : plugins.values()) {
            if (plugin.getType() == PluginType.DOCUMENT_REVIEW
                    && plugin instanceof DocumentReviewPlugin documentPlugin
                    && isEnabled(plugin)
                    && documentPlugin.isAvailable()) {
                result.add(documentPlugin);
            }
        }
        return result;
    }

    public List<RecommendationPlugin> enabledRecommendationPlugins() {
        List<RecommendationPlugin> result = new ArrayList<>();
        for (ForumPlugin plugin : plugins.values()) {
            if (plugin.getType() == PluginType.RECOMMENDATION
                    && plugin instanceof RecommendationPlugin recommendationPlugin
                    && isEnabled(plugin)) {
                result.add(recommendationPlugin);
            }
        }
        return result;
    }

    public void setEnabled(String key, boolean enabled) {
        ForumPlugin plugin = plugins.get(key);
        if (plugin == null) {
            throw ApiException.notFound("插件不存在: " + key);
        }
        overrides.put(key, enabled);
    }
}
