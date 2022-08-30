/*!
 *  Copyright (c) 2015 by Contributors
 * \file logging.h
 * \brief defines logging macros of dmlc
 *  allows use of GLOG, fall back to internal
 *  implementation when disabled
 */
#ifndef DMLC_LOGGING_H_
#define DMLC_LOGGING_H_
#include <cstdio>
#include <cstdlib>
#include <string>
#include <vector>
#include <stdexcept>
#include <memory>
#include "./base.h"

#if DMLC_LOG_STACK_TRACE
#include <cxxabi.h>
#include <sstream>
#include DMLC_EXECINFO_H
#endif

namespace dmlc {
/*!
 * \brief exception class that will be thrown by
 *  default logger if DMLC_LOG_FATAL_THROW == 1
 */
struct Error : public std::runtime_error {
  /*!
   * \brief constructor
   * \param s the error message
   */
  explicit Error(const std::string &s) : std::runtime_error(s) {}
};

#if DMLC_LOG_STACK_TRACE
// get stack trace logging depth from env variable.
inline size_t LogStackTraceLevel() {
  size_t level;
  if (auto var = std::getenv("DMLC_LOG_STACK_TRACE_DEPTH")) {
    if (1 == sscanf(var, "%zu", &level)) {
      return level + 1;
    }
  }
  return DMLC_LOG_STACK_TRACE_SIZE;
}

inline std::string Demangle(char const *msg_str) {
  using std::string;
  string msg(msg_str);
  size_t symbol_start = string::npos;
  size_t symbol_end = string::npos;
  if ( ((symbol_start = msg.find("_Z")) != string::npos)
       && (symbol_end = msg.find_first_of(" +", symbol_start)) ) {
    string left_of_symbol(msg, 0, symbol_start);
    string symbol(msg, symbol_start, symbol_end - symbol_start);
    string right_of_symbol(msg, symbol_end);

    int status = 0;
    size_t length = string::npos;
    std::unique_ptr<char, void (*)(void *__ptr)> demangled_symbol =
        {abi::__cxa_demangle(symbol.c_str(), 0, &length, &status), &std::free};
    if (demangled_symbol && status == 0 && length > 0) {
      string symbol_str(demangled_symbol.get());
      std::ostringstream os;
      os << left_of_symbol << symbol_str << right_of_symbol;
      return os.str();
    }
  }
  return string(msg_str);
}

// By default skip the first frame because
// that belongs to ~LogMessageFatal
inline std::string StackTrace(
    size_t start_frame = 1,
    const size_t stack_size = DMLC_LOG_STACK_TRACE_SIZE) {
  using std::string;
  std::ostringstream stacktrace_os;
  std::vector<void*> stack(stack_size);
  int nframes = backtrace(stack.data(), static_cast<int>(stack_size));
  if (start_frame < static_cast<size_t>(nframes)) {
    stacktrace_os << "Stack trace:\n";
  }
  char **msgs = backtrace_symbols(stack.data(), nframes);
  if (msgs != nullptr) {
    for (int frameno = start_frame; frameno < nframes; ++frameno) {
      string msg = dmlc::Demangle(msgs[frameno]);
      stacktrace_os << "  [bt] (" << frameno - start_frame << ") " << msg << "\n";
    }
  }
  free(msgs);
  string stack_trace = stacktrace_os.str();
  return stack_trace;
}

#else  // DMLC_LOG_STACK_TRACE is off

inline size_t LogStackTraceLevel() {
  return 0;
}

inline std::string demangle(char const* msg_str) {
  return std::string();
}

inline std::string StackTrace(size_t start_frame = 1,
                              const size_t stack_size = 0) {
  return std::string("Stack trace not available when "
  "DMLC_LOG_STACK_TRACE is disabled at compile time.");
}

#endif  // DMLC_LOG_STACK_TRACE
}  // namespace dmlc

#if DMLC_USE_GLOG
#include <glog/logging.h>

namespace dmlc {
/*!
 * \brief optionally redirect to google's init log
 * \param argv0 The arguments.
 */
inline void InitLogging(const char* argv0) {
  google::InitGoogleLogging(argv0);
}
}  // namespace dmlc

#elif defined DMLC_USE_LOGGING_LIBRARY

#include DMLC_USE_LOGGING_LIBRARY
namespace dmlc {
inline void InitLogging(const char*) {
  // DO NOTHING
}
}

#else
// use a light version of glog
#include <assert.h>
#include <iostream>
#include <sstream>
#include <ctime>

#if defined(_MSC_VER)
#pragma warning(disable : 4722)
#pragma warning(disable : 4068)
#endif

namespace dmlc {
inline void InitLogging(const char*) {
  // DO NOTHING
}

// get debug option from env variable.
inline bool DebugLoggingEnabled() {
  static int state = 0;
  if (state == 0) {
    if (auto var = std::getenv("DMLC_LOG_DEBUG")) {
      if (std::string(var) == "1") {
        state = 1;
      } else {
        state = -1;
      }
    } else {
      // by default hide debug logging.
      state = -1;
    }
  }
  return state == 1;
}

#ifndef DMLC_GLOG_DEFINED

template <typename X, typename Y>
std::unique_ptr<std::string> LogCheckFormat(const X& x, const Y& y) {
  std::ostringstream os;
  os << " (" << x << " vs. " << y << ") "; /* CHECK_XX(x, y) requires x and y can be serialized to string. Use CHECK(x OP y) otherwise. NOLINT(*) */
  // no std::make_unique until c++14
  return std::unique_ptr<std::string>(new std::string(os.str()));
}

// This function allows us to ignore sign comparison in the right scope.
#define DEFINE_CHECK_FUNC(name, op)                                                        \
  template <typename X, typename Y>                                                        \
  DMLC_ALWAYS_INLINE std::unique_ptr<std::string> LogCheck##name(const X& x, const Y& y) { \
    if (x op y) return nullptr;                                                            \
    return LogCheckFormat(x, y);                                                           \
  }                                                                                        \
  DMLC_ALWAYS_INLINE std::unique_ptr<std::string> LogCheck##name(int x, int y) {           \
    return LogCheck##name<int, int>(x, y);                                                 \
  }

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wsign-compare"
DEFINE_CHECK_FUNC(_LT, <)
DEFINE_CHECK_FUNC(_GT, >)
DEFINE_CHECK_FUNC(_LE, <=)
DEFINE_CHECK_FUNC(_GE, >=)
DEFINE_CHECK_FUNC(_EQ, ==)
DEFINE_CHECK_FUNC(_NE, !=)
#pragma GCC diagnostic pop

#define CHECK_BINARY_OP(name, op, x, y)                  \
  if (auto __dmlc__log__err = dmlc::LogCheck##name(x, y))  \
      dmlc::LogMessageFatal(__FILE__, __LINE__).stream() \
        << "Check failed: " << #x " " #op " " #y << *__dmlc__log__err << ": "

// Always-on checking
#define CHECK(x)                                           \
  if (!(x))                                                \
    dmlc::LogMessageFatal(__FILE__, __LINE__).stream()     \
      << "Check failed: " #x << ": "
#define CHECK_LT(x, y) CHECK_BINARY_OP(_LT, <, x, y)
#define CHECK_GT(x, y) CHECK_BINARY_OP(_GT, >, x, y)
#define CHECK_LE(x, y) CHECK_BINARY_OP(_LE, <=, x, y)
#define CHECK_GE(x, y) CHECK_BINARY_OP(_GE, >=, x, y)
#define CHECK_EQ(x, y) CHECK_BINARY_OP(_EQ, ==, x, y)
#define CHECK_NE(x, y) CHECK_BINARY_OP(_NE, !=, x, y)
#define CHECK_NOTNULL(x) \
  ((x) == NULL ? dmlc::LogMessageFatal(__FILE__, __LINE__).stream() << "Check  notnull: "  #x << ' ', (x) : (x)) // NOLINT(*)

// Debug-only checking.
#if DMLC_LOG_DEBUG
#define DCHECK(x) \
  while (false) CHECK(x)
#define DCHECK_LT(x, y) \
  while (false) CHECK((x) < (y))
#define DCHECK_GT(x, y) \
  while (false) CHECK((x) > (y))
#define DCHECK_LE(x, y) \
  while (false) CHECK((x) <= (y))
#define DCHECK_GE(x, y) \
  while (false) CHECK((x) >= (y))
#define DCHECK_EQ(x, y) \
  while (false) CHECK((x) == (y))
#define DCHECK_NE(x, y) \
  while (false) CHECK((x) != (y))
#else
#define DCHECK(x) CHECK(x)
#define DCHECK_LT(x, y) CHECK((x) < (y))
#define DCHECK_GT(x, y) CHECK((x) > (y))
#define DCHECK_LE(x, y) CHECK((x) <= (y))
#define DCHECK_GE(x, y) CHECK((x) >= (y))
#define DCHECK_EQ(x, y) CHECK((x) == (y))
#define DCHECK_NE(x, y) CHECK((x) != (y))
#endif  // DMLC_LOG_DEBUG

#if DMLC_LOG_CUSTOMIZE
#define LOG_INFO dmlc::CustomLogMessage(__FILE__, __LINE__)
#else
#define LOG_INFO dmlc::LogMessage(__FILE__, __LINE__)
#endif
#define LOG_ERROR LOG_INFO
#define LOG_WARNING LOG_INFO
#define LOG_FATAL dmlc::LogMessageFatal(__FILE__, __LINE__)
#define LOG_QFATAL LOG_FATAL

// Poor man version of VLOG
#define VLOG(x) LOG_INFO.stream()

#define LOG(severity) LOG_##severity.stream()
#define LG LOG_INFO.stream()
#define LOG_IF(severity, condition) \
  !(condition) ? (void)0 : dmlc::LogMessageVoidify() & LOG(severity)

#if DMLC_LOG_DEBUG

#define LOG_DFATAL LOG_FATAL
#define DFATAL FATAL
#define DLOG(severity) LOG_IF(severity, ::dmlc::DebugLoggingEnabled())
#define DLOG_IF(severity, condition) LOG_IF(severity, ::dmlc::DebugLoggingEnabled() && (condition))

#else

#define LOG_DFATAL LOG_ERROR
#define DFATAL ERROR
#define DLOG(severity) true ? (void)0 : dmlc::LogMessageVoidify() & LOG(severity)
#define DLOG_IF(severity, condition) \
  (true || !(condition)) ? (void)0 : dmlc::LogMessageVoidify() & LOG(severity)
#endif

// Poor man version of LOG_EVERY_N
#define LOG_EVERY_N(severity, n) LOG(severity)

#endif  // DMLC_GLOG_DEFINED

class DateLogger {
 public:
  DateLogger() {
#if defined(_MSC_VER)
    _tzset();
#endif
  }
  const char* HumanDate() {
#if !defined(_LIBCPP_SGX_CONFIG) && DMLC_LOG_NODATE == 0
#if defined(_MSC_VER)
    _strtime_s(buffer_, sizeof(buffer_));
#else
    time_t time_value = time(NULL);
    struct tm *pnow;
#if !defined(_WIN32)
    struct tm now;
    pnow = localtime_r(&time_value, &now);
#else
    pnow = localtime(&time_value);  // NOLINT(*)
#endif
    snprintf(buffer_, sizeof(buffer_), "%02d:%02d:%02d",
             pnow->tm_hour, pnow->tm_min, pnow->tm_sec);
#endif
    return buffer_;
#else
    return "";
#endif  // _LIBCPP_SGX_CONFIG
  }

 private:
  char buffer_[9];
};

#ifndef _LIBCPP_SGX_NO_IOSTREAMS
class LogMessage {
 public:
  LogMessage(const char* file, int line)
      :
#ifdef __ANDROID__
        log_stream_(std::cout)
#else
        log_stream_(std::cerr)
#endif
  {
    log_stream_ << "[" << pretty_date_.HumanDate() << "] " << file << ":"
                << line << ": ";
  }
  ~LogMessage() { log_stream_ << '\n'; }
  std::ostream& stream() { return log_stream_; }

 protected:
  std::ostream& log_stream_;

 private:
  DateLogger pretty_date_;
  LogMessage(const LogMessage&);
  void operator=(const LogMessage&);
};

// customized logger that can allow user to define where to log the message.
class CustomLogMessage {
 public:
  CustomLogMessage(const char* file, int line) {
    log_stream_ << "[" << DateLogger().HumanDate() << "] " << file << ":"
                << line << ": ";
  }
  ~CustomLogMessage() {
    Log(log_stream_.str());
  }
  std::ostream& stream() { return log_stream_; }
  /*!
   * \brief customized logging of the message.
   * This function won't be implemented by libdmlc
   * \param msg The message to be logged.
   */
  static void Log(const std::string& msg);

 private:
  std::ostringstream log_stream_;
};
#else
class DummyOStream {
 public:
  template <typename T>
  DummyOStream& operator<<(T _) { return *this; }
  inline std::string str() { return ""; }
};
class LogMessage {
 public:
  LogMessage(const char* file, int line) : log_stream_() {}
  DummyOStream& stream() { return log_stream_; }

 protected:
  DummyOStream log_stream_;

 private:
  LogMessage(const LogMessage&);
  void operator=(const LogMessage&);
};
#endif


#if defined(_LIBCPP_SGX_NO_IOSTREAMS)
class LogMessageFatal : public LogMessage {
 public:
  LogMessageFatal(const char* file, int line) : LogMessage(file, line) {}
  ~LogMessageFatal() {
    abort();
  }
 private:
  LogMessageFatal(const LogMessageFatal&);
  void operator=(const LogMessageFatal&);
};
#elif DMLC_LOG_FATAL_THROW == 0
class LogMessageFatal : public LogMessage {
 public:
  LogMessageFatal(const char* file, int line) : LogMessage(file, line) {}
  ~LogMessageFatal() {
    log_stream_ << "\n" << StackTrace(1, LogStackTraceLevel()) << "\n";
    abort();
  }

 private:
  LogMessageFatal(const LogMessageFatal&);
  void operator=(const LogMessageFatal&);
};
#else
class LogMessageFatal {
 public:
  LogMessageFatal(const char *file, int line) {
    GetEntry().Init(file, line);
  }
  std::ostringstream &stream() { return GetEntry().log_stream; }
  DMLC_NO_INLINE ~LogMessageFatal() DMLC_THROW_EXCEPTION {
#if DMLC_LOG_STACK_TRACE
    GetEntry().log_stream << "\n"
                          << StackTrace(1, LogStackTraceLevel())
                          << "\n";
#endif
    throw GetEntry().Finalize();
  }

 private:
  struct Entry {
    std::ostringstream log_stream;
    DMLC_NO_INLINE void Init(const char *file, int line) {
      DateLogger date;
      log_stream.str("");
      log_stream.clear();
      log_stream << "[" << date.HumanDate() << "] " << file << ":" << line
                 << ": ";
    }
    dmlc::Error Finalize() {
#if DMLC_LOG_BEFORE_THROW
      LOG(ERROR) << log_stream.str();
#endif
      return dmlc::Error(log_stream.str());
    }
    // Due to a bug in MinGW, objects with non-trivial destructor cannot be thread-local.
    // See https://sourceforge.net/p/mingw-w64/bugs/527/
    // Hence, don't use thread-local for the log stream if the compiler is MinGW.
#if !(defined(__MINGW32__) || defined(__MINGW64__))
    DMLC_NO_INLINE static Entry& ThreadLocal() {
      static thread_local Entry result;
      return result;
    }
#endif
  };
  LogMessageFatal(const LogMessageFatal &);
  void operator=(const LogMessageFatal &);

#if defined(__MINGW32__) || defined(__MINGW64__)
  DMLC_NO_INLINE Entry& GetEntry() {
    return entry_;
  }

  Entry entry_;
#else
  DMLC_NO_INLINE Entry& GetEntry() {
    return Entry::ThreadLocal();
  }
#endif
};
#endif

// This class is used to explicitly ignore values in the conditional
// logging macros.  This avoids compiler warnings like "value computed
// is not used" and "statement has no effect".
class LogMessageVoidify {
 public:
  LogMessageVoidify() {}
  // This has to be an operator with a precedence lower than << but
  // higher than "?:". See its usage.
#if !defined(_LIBCPP_SGX_NO_IOSTREAMS)
  void operator&(std::ostream&) {}
#endif
};

}  // namespace dmlc

#endif
#endif  // DMLC_LOGGING_H_
