package com.openforum.dto.authentication;

import java.util.List;

public record AuthLevelMetaDTO(
        String level,
        String label,
        List<String> requirements,
        List<String> tips) {
}
