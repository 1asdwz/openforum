package com.openforum.plugin;

import com.openforum.common.JsonSupport;
import com.openforum.config.OpenForumAiProperties;
import com.openforum.entity.enums.AuthStatus;
import com.openforum.entity.enums.AuthType;
import com.openforum.entity.enums.ReviewMethod;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.core.ParameterizedTypeReference;
import org.springframework.http.HttpHeaders;
import org.springframework.http.MediaType;
import org.springframework.http.client.SimpleClientHttpRequestFactory;
import org.springframework.stereotype.Component;
import org.springframework.web.client.RestClient;

import java.time.Duration;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

@Component
public class AiReviewPlugin implements ReviewPlugin {

    private static final Logger log = LoggerFactory.getLogger(AiReviewPlugin.class);

    private static final String SYSTEM_PROMPT = """
            你是用户认证审核助手。请根据给定的认证类型与认证数据判断是否通过。
            只返回严格 JSON，不要任何多余文字，格式：
            {"status":"APPROVED|REJECTED|PENDING","confidence":0.0-1.0,"reason":"简短说明"}
            资料明显缺失或格式错误应返回 REJECTED；无法判断时返回 PENDING。
            """;

    private final OpenForumAiProperties properties;
    private final JsonSupport jsonSupport;
    private final RestClient restClient;

    public AiReviewPlugin(OpenForumAiProperties properties, JsonSupport jsonSupport) {
        this.properties = properties;
        this.jsonSupport = jsonSupport;
        SimpleClientHttpRequestFactory factory = new SimpleClientHttpRequestFactory();
        Duration timeout = Duration.ofSeconds(properties.timeoutSeconds());
        factory.setConnectTimeout(timeout);
        factory.setReadTimeout(timeout);
        this.restClient = RestClient.builder().requestFactory(factory).build();
    }

    @Override
    public String getKey() {
        return "ai-review";
    }

    @Override
    public String getName() {
        return "AI 智能审核";
    }

    @Override
    public PluginType getType() {
        return PluginType.REVIEW;
    }

    @Override
    public boolean isEnabledByDefault() {
        return false;
    }

    @Override
    public String getDescription() {
        return "调用 OpenAI 兼容接口进行智能审核，未配置密钥时自动回退规则引擎";
    }

    @Override
    public boolean isAvailable() {
        return properties.isConfigured();
    }

    @Override
    public ReviewMethod getReviewMethod() {
        return ReviewMethod.AI;
    }

    @Override
    public ReviewResult review(AuthType type, Map<String, Object> data) {
        if (!isAvailable()) {
            return new ReviewResult(AuthStatus.PENDING, 0.0, "AI 未配置，跳过");
        }
        try {
            Map<String, Object> systemMessage = Map.of("role", "system", "content", SYSTEM_PROMPT);
            Map<String, Object> userMessage = Map.of("role", "user",
                    "content", "认证类型: " + type.name() + "\n认证数据: " + jsonSupport.toJson(data));
            Map<String, Object> requestBody = new LinkedHashMap<>();
            requestBody.put("model", properties.model());
            requestBody.put("temperature", 0);
            requestBody.put("messages", List.of(systemMessage, userMessage));

            Map<String, Object> response = restClient.post()
                    .uri(chatCompletionsUrl())
                    .header(HttpHeaders.AUTHORIZATION, "Bearer " + properties.apiKey())
                    .contentType(MediaType.APPLICATION_JSON)
                    .body(requestBody)
                    .retrieve()
                    .body(new ParameterizedTypeReference<Map<String, Object>>() {
                    });

            String content = extractContent(response);
            if (content == null || content.isBlank()) {
                return new ReviewResult(AuthStatus.PENDING, 0.0, "AI 返回内容为空");
            }
            Map<String, Object> parsed = jsonSupport.toMap(stripCodeFence(content));
            AuthStatus status = switch (String.valueOf(parsed.getOrDefault("status", "PENDING")).toUpperCase()) {
                case "APPROVED" -> AuthStatus.APPROVED;
                case "REJECTED" -> AuthStatus.REJECTED;
                default -> AuthStatus.PENDING;
            };
            double confidence = parseConfidence(parsed.get("confidence"));
            Object reason = parsed.get("reason");
            return new ReviewResult(status, confidence, reason == null ? "AI 审核" : String.valueOf(reason));
        } catch (Exception ex) {
            log.warn("AI review failed, falling back to other plugins: {}", ex.getMessage());
            return new ReviewResult(AuthStatus.PENDING, 0.0, "AI 审核不可用");
        }
    }

    private String chatCompletionsUrl() {
        String base = properties.apiBase();
        while (base.endsWith("/")) {
            base = base.substring(0, base.length() - 1);
        }
        return base + "/chat/completions";
    }

    @SuppressWarnings("unchecked")
    private String extractContent(Map<String, Object> response) {
        if (response == null) {
            return null;
        }
        Object choicesValue = response.get("choices");
        if (!(choicesValue instanceof List<?> choices) || choices.isEmpty()) {
            return null;
        }
        Object firstValue = choices.get(0);
        if (!(firstValue instanceof Map<?, ?> first)) {
            return null;
        }
        Object messageValue = first.get("message");
        if (!(messageValue instanceof Map<?, ?> message)) {
            return null;
        }
        Object contentValue = message.get("content");
        return contentValue == null ? null : String.valueOf(contentValue);
    }

    private String stripCodeFence(String content) {
        String trimmed = content.trim();
        if (trimmed.startsWith("```")) {
            int firstNewline = trimmed.indexOf('\n');
            if (firstNewline > 0) {
                trimmed = trimmed.substring(firstNewline + 1);
            }
            int lastFence = trimmed.lastIndexOf("```");
            if (lastFence >= 0) {
                trimmed = trimmed.substring(0, lastFence);
            }
        }
        return trimmed.trim();
    }

    private double parseConfidence(Object value) {
        if (value instanceof Number number) {
            return number.doubleValue();
        }
        if (value == null) {
            return 0.0;
        }
        try {
            return Double.parseDouble(String.valueOf(value));
        } catch (NumberFormatException ex) {
            return 0.0;
        }
    }
}
