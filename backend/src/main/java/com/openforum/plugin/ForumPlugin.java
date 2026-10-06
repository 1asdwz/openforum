package com.openforum.plugin;

/**
 * Backend SPI. Every Spring bean implementing this interface is auto-collected by
 * {@link PluginRegistry}.
 */
public interface ForumPlugin {

    String getKey();

    String getName();

    PluginType getType();

    boolean isEnabledByDefault();

    String getDescription();
}
