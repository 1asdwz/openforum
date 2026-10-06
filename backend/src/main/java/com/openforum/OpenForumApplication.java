package com.openforum;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.boot.context.properties.ConfigurationPropertiesScan;

@SpringBootApplication
@ConfigurationPropertiesScan
public class OpenForumApplication {

    public static void main(String[] args) {
        SpringApplication.run(OpenForumApplication.class, args);
    }
}
