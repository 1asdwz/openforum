package com.openforum.security;

import com.openforum.common.JsonSupport;
import com.openforum.config.OpenForumGoogleProperties;
import com.openforum.exception.ApiException;
import org.springframework.http.client.SimpleClientHttpRequestFactory;
import org.springframework.stereotype.Component;
import org.springframework.web.client.RestClient;
import org.springframework.web.client.RestClientException;

import java.time.Duration;
import java.util.Arrays;
import java.util.Collection;
import java.util.List;
import java.util.Map;

/**
 * 校验 Google ID Token（通过 Google 官方 tokeninfo 端点）。
 */
@Component
public class GoogleTokenVerifier {

    private static final String TOKEN_INFO_URL = "https://oauth2.googleapis.com/tokeninfo";

    private final OpenForumGoogleProperties properties;
    private final JsonSupport jsonSupport;
    private final RestClient restClient;

    public GoogleTokenVerifier(OpenForumGoogleProperties properties, JsonSupport jsonSupport) {
        this.properties = properties;
        this.jsonSupport = jsonSupport;
        SimpleClientHttpRequestFactory factory = new SimpleClientHttpRequestFactory();
        Duration timeout = Duration.ofSeconds(6);
        factory.setConnectTimeout(timeout);
        factory.setReadTimeout(timeout);
        this.restClient = RestClient.builder().requestFactory(factory).build();
    }

    public GoogleUser verify(String idToken) {
        if (!properties.isConfigured()) {
            throw ApiException.badRequest("Google 登录未配置");
        }
        if (idToken == null || idToken.isBlank()) {
            throw ApiException.unauthorized("Google 登录凭证无效");
        }

        String body;
        try {
            body = restClient.get()
                    .uri(TOKEN_INFO_URL + "?id_token={token}", idToken)
                    .retrieve()
                    .body(String.class);
        } catch (RestClientException ex) {
            throw ApiException.unauthorized("Google 登录凭证无效");
        }
        if (body == null || body.isBlank()) {
            throw ApiException.unauthorized("Google 登录凭证无效");
        }

        Map<String, Object> claims = jsonSupport.toMap(body);

        if (!audienceMatches(claims.get("aud"))) {
            throw ApiException.unauthorized("Google 登录凭证无效");
        }

        Object emailVerified = claims.get("email_verified");
        boolean verified = emailVerified instanceof Boolean value
                ? value
                : "true".equalsIgnoreCase(String.valueOf(emailVerified));
        if (!verified) {
            throw ApiException.badRequest("Google 账号邮箱未验证");
        }

        Object sub = claims.get("sub");
        if (sub == null || String.valueOf(sub).isBlank()) {
            throw ApiException.unauthorized("Google 登录凭证无效");
        }

        return new GoogleUser(
                String.valueOf(sub),
                stringClaim(claims, "email"),
                stringClaim(claims, "name"),
                stringClaim(claims, "picture"));
    }

    private boolean audienceMatches(Object aud) {
        List<String> allowed = Arrays.stream(properties.clientId().split(","))
                .map(String::trim)
                .filter(value -> !value.isEmpty())
                .toList();
        if (allowed.isEmpty()) {
            return false;
        }
        if (aud instanceof Collection<?> audiences) {
            return audiences.stream().map(String::valueOf).anyMatch(allowed::contains);
        }
        return aud != null && allowed.contains(String.valueOf(aud));
    }

    private String stringClaim(Map<String, Object> claims, String key) {
        Object value = claims.get(key);
        return value == null ? null : String.valueOf(value);
    }

    public record GoogleUser(String googleId, String email, String name, String picture) {
    }
}
