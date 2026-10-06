package com.openforum.repository;

import com.openforum.entity.AuthMedia;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.Optional;

public interface AuthMediaRepository extends JpaRepository<AuthMedia, Long> {

    List<AuthMedia> findByUserIdAndIdIn(Long userId, List<Long> ids);

    Optional<AuthMedia> findByIdAndUserId(Long id, Long userId);
}
