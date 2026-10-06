package com.openforum.plugin;

import com.openforum.common.JsonSupport;
import com.openforum.config.OpenForumAiProperties;
import com.openforum.entity.enums.AuthStatus;
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
import java.util.ArrayList;
import java.util.Base64;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * 证件/自拍图片的多模态 AI 审核（OpenAI 兼容视觉端点，复用 qwen 配置）。
 */
@Component
public class AiVisionReviewPlugin implements DocumentReviewPlugin {

    private static final Logger log = LoggerFactory.getLogger(AiVisionReviewPlugin.class);

    private static final String SYSTEM_PROMPT = """
            你是用户认证审核助手。我会给你用户的认证类型、声明的字段以及证件/自拍照片。
            请判断材料是否真实、清晰、与声明一致，并尽量抽取关键字段。
            只返回严格 JSON，不要任何多余文字，格式：
            {"status":"APPROVED|REJECTED|PENDING","confidence":0.0-1.0,"reason":"简短说明",
             "extracted":{"fullName":"","idNumber":""}}
             - 图片与声明一致且清晰：APPROVED，confidence 0.85 以上。
            - 图片模糊、遮挡、明显伪造或与声明不符：REJECTED。
            - 无法判断：PENDING。
            """;

    private final OpenForumAiProperties properties;
    private final JsonSupport jsonSupport;
    private final RestClient restClient;

    public AiVisionReviewPlugin(OpenForumAiProperties properties, JsonSupport jsonSupport) {
        this.properties = properties;
        this.jsonSupport = jsonSupport;
        SimpleClientHttpRequestFactory factory = new SimpleClientHttpRequestFactory();
        Duration timeout = Duration.ofSeconds(Math.max(properties.timeoutSeconds(), 30));
        factory.setConnectTimeout(timeout);
        factory.setReadTimeout(timeout);
        this.restClient = RestClient.builder().requestFactory(factory).build();
    }

    @Override
    public String getKey() {
        return "ai-vision-review";
    }

    @Override
    public String getName() {
        return "AI 证件视觉审核";
    }

    @Override
    public PluginType getType() {
        return PluginType.DOCUMENT_REVIEW;
    }

    @Override
    public boolean isEnabledByDefault() {
        return false;
    }

    @Override
    public String getDescription() {
        return "调用多模态大模型审阅证件图片并抽取实名信息，未配置密钥时自动跳过";
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
    public ReviewResult reviewDocuments(DocumentReviewContext ctx) {
        if (!isAvailable()) {
            return new ReviewResult(AuthStatus.PENDING, 0.0, "AI 未配置，跳过");
        }
        try {
            List<Map<String, Object>> content = new ArrayList<>();
            content.add(multipartText("认证类型: " + ctx.type().name()
                    + "\n认证等级: " + ctx.level().name()
                    + "\n声明字段: " + jsonSupport.toJson(ctx.data())));
            int index = 0;
            for (MediaRef document : ctx.documents()) {
                index++;
                content.add(multipartText("证件图片 #" + index + "（" + document.contentType() + "）"));
                content.add(imagePart(document.contentType(), document.bytes()));
            }
            if (ctx.videoPoster() != null && ctx.videoPoster().length > 0) {
                content.add(multipartText("视频首帧（活体录制画面）"));
                content.add(imagePart("image/jpeg", ctx.videoPoster()));
            }

            Map<String, Object> systemMessage = Map.of("role", "system", "content", SYSTEM_PROMPT);
            Map<String, Object> userMessage = Map.of("role", "user", "content", content);

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

            String raw = extractContent(response);
            if (raw == null || raw.isBlank()) {
                return new ReviewResult(AuthStatus.PENDING, 0.0, "AI 返回内容为空");
            }
            Map<String, Object> parsed = jsonSupport.toMap(stripCodeFence(raw));
            AuthStatus status = switch (String.valueOf(parsed.getOrDefault("status", "PENDING")).toUpperCase()) {
                case "APPROVED" -> AuthStatus.APPROVED;
                case "REJECTED" -> AuthStatus.REJECTED;
                default -> AuthStatus.PENDING;
            };
            double confidence = parseConfidence(parsed.get("confidence"));
            Object reason = parsed.get("reason");
            Map<String, Object> extracted = asMap(parsed.get("extracted"));
            return new ReviewResult(status, confidence,
                    reason == null ? "AI 视觉审核" : String.valueOf(reason), extracted);
        } catch (Exception ex) {
            log.warn("AI vision review failed: {}", ex.getMessage());
            return new ReviewResult(AuthStatus.PENDING, 0.0, "AI 视觉审核不可用");
        }
    }

    private Map<String, Object> multipartText(String text) {
        return Map.of("type", "text", "text", text);
    }

    private Map<String, Object> imagePart(String contentType, byte[] bytes) {
        String mime = (contentType == null || contentType.isBlank()) ? "image/jpeg" : contentType;
        String dataUrl = "data:" + mime + ";base64," + Base64.getEncoder().encodeToString(bytes);
        return Map.of("type", "image_url", "image_url", Map.of("url", dataUrl));
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
        if (!(choices.get(0) instanceof Map<?, ?> first)) {
            return null;
        }
        Object messageValue = first.get("message");
        if (!(messageValue instanceof Map<?, ?> message)) {
            return null;
        }
        Object contentValue = message.get("content");
        return contentValue == null ? null : String.valueOf(contentValue);
    }

    private Map<String, Object> asMap(Object value) {
        if (value instanceof Map<?, ?> map) {
            Map<String, Object> result = new LinkedHashMap<>();
            map.forEach((k, v) -> result.put(String.valueOf(k), v));
            return result;
        }
        return Map.of();
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
