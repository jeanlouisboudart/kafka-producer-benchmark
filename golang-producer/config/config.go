package config

import (
	"fmt"
	"strings"

	"github.com/spf13/viper"
)

type Config struct {
	NB_TOPICS                 int    `mapstructure:"NB_TOPICS"`
	REPLICATION_FACTOR        int    `mapstructure:"REPLICATION_FACTOR"`
	MESSAGE_SIZE              int    `mapstructure:"MESSAGE_SIZE"`
	NB_MESSAGES               int    `mapstructure:"NB_MESSAGES"`
	REPORTING_INTERVAL        int    `mapstructure:"REPORTING_INTERVAL"`
	USE_RANDOM_KEYS           bool   `mapstructure:"USE_RANDOM_KEYS"`
	AGG_PER_TOPIC_NB_MESSAGES int    `mapstructure:"AGG_PER_TOPIC_NB_MESSAGES"`
	TOPIC_PREFIX              string `mapstructure:"TOPIC_PREFIX"`

	KAFKA_CONFIGS map[string]interface{}
}

// LoadConfig reads configuration from file or environment variables.
func LoadConfig(path string) (config Config, err error) {
	viper.SetConfigFile(path)  // Path to look for the config file.
	viper.SetConfigType("env") // format of config file (yaml, json, properties, etc.)
	viper.AutomaticEnv()

	err = viper.ReadInConfig() // Find and read the config file
	if err != nil {            // Handle errors reading the config file
		return
	}

	config = Config{}
	err = viper.Unmarshal(&config) // Unmarshal the config into a struct
	if err != nil {
		return config, err
	}

	config.KAFKA_CONFIGS = make(map[string]interface{})

	for k, v := range viper.AllSettings() {
		if strings.HasPrefix(k, "kafka_") {
			key := strings.Replace(k, "kafka_", "", -1)
			key = strings.Replace(key, "_", ".", -1)
			config.KAFKA_CONFIGS[key] = v.(string)
			fmt.Println(config.KAFKA_CONFIGS)
		}
	}

	return config, nil
}
