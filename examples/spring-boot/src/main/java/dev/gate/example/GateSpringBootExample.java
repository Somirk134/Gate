package dev.gate.example;

import java.time.Instant;
import java.util.Map;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestHeader;
import org.springframework.web.bind.annotation.RestController;

@SpringBootApplication
public class GateSpringBootExample {
    public static void main(String[] args) {
        SpringApplication.run(GateSpringBootExample.class, args);
    }
}

@RestController
class ExampleController {
    @GetMapping("/api/health")
    Map<String, Object> health(@RequestHeader(value = "Host", required = false) String host) {
        return Map.of(
            "ok", true,
            "service", "spring-boot",
            "host", host == null ? "" : host,
            "time", Instant.now().toString()
        );
    }

    @PostMapping("/api/echo")
    Map<String, Object> echo(@RequestBody Map<String, Object> body) {
        return Map.of("received", body);
    }
}
