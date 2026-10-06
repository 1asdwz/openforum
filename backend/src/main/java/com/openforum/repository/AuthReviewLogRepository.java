package com.openforum.repository;

import com.openforum.entity.AuthReviewLog;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;

public interface AuthReviewLogRepository extends JpaRepository<AuthReviewLog, Long> {

    List<AuthReviewLog> findByAuthenticationIdOrderByCreatedAtAsc(Long authenticationId);
}
