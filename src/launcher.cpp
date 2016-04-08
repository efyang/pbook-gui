#include <string>
#include <unistd.h>

int main(int numArgs, char* args[])
{
    std::string aux(args[0]);

#if defined(_WIN32) || defined(WIN32)
    int pos = aux.rfind('\\');
    std::string exe = std::string("bin\\pbook-gui.exe");
#else
    int pos = aux.rfind('/');
    std::string exe = std::string("bin/pbook-gui");
#endif
    std::string path = aux.substr(0, pos + 1);
    path += exe;
    execl(path.c_str(), path.c_str(), NULL);
    return 0;
}
