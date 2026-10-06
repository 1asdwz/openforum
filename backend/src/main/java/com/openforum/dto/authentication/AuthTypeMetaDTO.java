package com.openforum.dto.authentication;

import java.util.List;

public record AuthTypeMetaDTO(String type, String label, List<AuthFieldDTO> fields, List<AuthLevelMetaDTO> levels) {
}
