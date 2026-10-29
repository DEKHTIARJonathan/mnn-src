// Apache-2.0. Original arithmetic fixture; no external model assets.
#include <MNN/Interpreter.hpp>
#include <MNN/Tensor.hpp>
#include <MNN/expr/ExprCreator.hpp>
#include <cmath>
#include <cstring>
#include <memory>

extern "C" int smoke(const char* expected) {
    using namespace MNN::Express;
    if (std::strcmp(MNN::getVersion(), expected)) return 1;
    auto x = _Input({1, 1, 2, 2}, NCHW, halide_type_of<float>());
    x->setName("input");
    auto y = _Add(_Multiply(x, _Scalar<float>(2)), _Scalar<float>(1));
    y->setName("output");
    auto model = Variable::save({y});
    for (int iteration = 0; iteration < 3; ++iteration) {
        std::unique_ptr<MNN::Interpreter> engine(MNN::Interpreter::createFromBuffer(model.data(), model.size()));
        if (!engine) return 2;
        MNN::BackendConfig backend;
        backend.precision = MNN::BackendConfig::Precision_High;
        MNN::ScheduleConfig config;
        config.type = MNN_FORWARD_CPU;
        config.numThread = 1;
        config.backendConfig = &backend;
        auto session = engine->createSession(config);
        if (!session) return 3;
        auto input = engine->getSessionInput(session, "input");
        auto output = engine->getSessionOutput(session, "output");
        if (!input || !output) return 4;
        MNN::Tensor host_input(input, MNN::Tensor::CAFFE);
        MNN::Tensor host_output(output, MNN::Tensor::CAFFE);
        if (host_input.elementSize() != 4 || host_output.elementSize() != 4) return 5;
        engine->releaseModel();
        for (int run = 0; run < 2; ++run) {
            for (int i = 0; i < 4; ++i) host_input.host<float>()[i] = float(i - 2 + run);
            if (!input->copyFromHostTensor(&host_input)) return 6;
            if (engine->runSession(session) != MNN::NO_ERROR) return 7;
            if (!output->copyToHostTensor(&host_output)) return 8;
            for (int i = 0; i < 4; ++i) {
                float actual = host_output.host<float>()[i];
                if (!std::isfinite(actual) || std::fabs(actual - (2 * (i - 2 + run) + 1)) > 1e-6f) return 9;
            }
        }
    }
    return 0;
}
