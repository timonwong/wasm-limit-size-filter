package test

import (
	"istio.io/proxy/test/envoye2e/env"
)

var ExtensionE2ETests *env.TestInventory

func init() {
	// NOTE: 重要, 因为开启了 t.Parallel(), E2E driver 根据这里的测试名称生成端口号, 如果不显式指定, 可能造成端口冲突
	ExtensionE2ETests = &env.TestInventory{
		Tests: []string{
			"TestLimitSize/WrongConfig",
			"TestLimitSize/LimitRequest-1B-Fail",
			"TestLimitSize/LimitRequest-5B-CT-Fail",
			"TestLimitSize/LimitRequest-100B-Ok",
			"TestLimitSize/LimitResponse-1B-Fail",
			"TestLimitSize/LimitResponse-2B-Fail",
			"TestLimitSize/LimitResponse-100B-Ok",
		},
	}
}
